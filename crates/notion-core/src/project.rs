//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use lazycell::LazyCell;

use distro::node::load_default_npm_version;
use distro::DistroVersion;
use manifest::{serial, Manifest};
use notion_fail::{ExitCode, Fallible, NotionError, NotionFail, ResultExt};
use platform::PlatformSpec;
use shim;

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn is_node_modules(dir: &Path) -> bool {
    dir.file_name() == Some(OsStr::new("node_modules"))
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, |parent| is_node_modules(parent))
}

fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

pub struct LazyDependentBins {
    bins: LazyCell<HashMap<String, String>>,
}

impl LazyDependentBins {
    /// Constructs a new `LazyDependentBins`.
    pub fn new() -> LazyDependentBins {
        LazyDependentBins {
            bins: LazyCell::new(),
        }
    }

    /// Forces creating the dependent bins and returns an immutable reference to it.
    pub fn get(&self, project: &Project) -> Fallible<&HashMap<String, String>> {
        self.bins
            .try_borrow_with(|| Ok(project.dependent_binaries()?))
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Could not read dependent package info: {}", error)]
#[notion_fail(code = "FileSystemError")]
pub(crate) struct DepPackageReadError {
    pub(crate) error: String,
}

impl DepPackageReadError {
    pub(crate) fn from_error(error: &NotionError) -> Self {
        DepPackageReadError {
            error: error.to_string(),
        }
    }
}

/// Thrown when a user tries to pin a Yarn version before pinning a Node version.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "There is no pinned node version for this project")]
#[notion_fail(code = "ConfigurationError")]
pub(crate) struct NoPinnedNodeVersion;

impl NoPinnedNodeVersion {
    pub(crate) fn new() -> Self {
        NoPinnedNodeVersion
    }
}

/// Thrown when a user tries to `notion pin` something other than node/yarn/npm.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Only node, yarn, and npm can be pinned in a project")]
#[notion_fail(code = "InvalidArguments")]
pub(crate) struct CannotPinPackageError;

impl CannotPinPackageError {
    pub(crate) fn new() -> Self {
        CannotPinPackageError
    }
}

/// A Node project tree in the filesystem.
pub struct Project {
    manifest: Manifest,
    project_root: PathBuf,
    dependent_bins: LazyDependentBins,
}

impl Project {
    /// Returns the Node project containing the current working directory,
    /// if any.
    pub fn for_current_dir() -> Fallible<Option<Project>> {
        let current_dir: &Path = &env::current_dir().unknown()?;
        Self::for_dir(&current_dir)
    }

    /// Returns the Node project for the input directory, if any.
    pub fn for_dir(base_dir: &Path) -> Fallible<Option<Project>> {
        let mut dir = base_dir.clone();
        while !is_project_root(dir) {
            dir = match dir.parent() {
                Some(parent) => parent,
                None => {
                    return Ok(None);
                }
            }
        }

        Ok(Some(Project {
            manifest: Manifest::for_dir(&dir)?,
            project_root: PathBuf::from(dir),
            dependent_bins: LazyDependentBins::new(),
        }))
    }

    /// Returns the pinned platform image, if any.
    pub fn platform(&self) -> Option<Rc<PlatformSpec>> {
        self.manifest.platform()
    }

    /// Returns true if the project manifest contains a toolchain.
    pub fn is_pinned(&self) -> bool {
        self.manifest.platform().is_some()
    }

    /// Returns the project manifest (`package.json`) for this project.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Returns the path to the `package.json` file for this project.
    pub fn package_file(&self) -> PathBuf {
        self.project_root.join("package.json")
    }

    /// Returns the path to the local binary directory for this project.
    pub fn local_bin_dir(&self) -> PathBuf {
        let sub_dir: PathBuf = ["node_modules", ".bin"].iter().collect();
        self.project_root.join(sub_dir)
    }

    /// Returns true if the input binary name is a direct dependency of the input project
    pub fn has_direct_bin(&self, bin_name: &OsStr) -> Fallible<bool> {
        let dep_bins = self.dependent_bins.get(&self)?;
        if let Some(bin_name_str) = bin_name.to_str() {
            if dep_bins.contains_key(bin_name_str) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Automatically shim the binaries of all direct dependencies of this project and
    /// return a vector of any errors which occurred while doing so.
    pub fn autoshim(&self) -> Vec<NotionError> {
        let dependent_binaries = self.dependent_binary_names_fault_tolerant();
        let mut errors = Vec::new();

        for result in dependent_binaries {
            match result {
                Ok(name) => {
                    if let Err(error) = shim::create(&name) {
                        errors.push(error);
                    }
                }
                Err(error) => errors.push(error),
            }
        }

        errors
    }

    /// Returns a mapping of the names to paths for all the binaries installed
    /// by direct dependencies of the current project.
    fn dependent_binaries(&self) -> Fallible<HashMap<String, String>> {
        let mut dependent_bins = HashMap::new();
        let all_deps = Manifest::for_dir(&self.project_root)?.merged_dependencies();
        let all_dep_paths = all_deps.iter().map(|name| self.get_dependency_path(name));

        // use those project paths to get the "bin" info for each project
        for pkg_path in all_dep_paths {
            let pkg_info =
                Manifest::for_dir(&pkg_path).with_context(DepPackageReadError::from_error)?;
            let bin_map = pkg_info.bin;
            for (name, path) in bin_map.iter() {
                dependent_bins.insert(name.clone(), path.clone());
            }
        }
        Ok(dependent_bins)
    }

    /// Gets the names of the binaries of all direct dependencies and returns them along
    /// with any errors which occurred while doing so.
    fn dependent_binary_names_fault_tolerant(&self) -> Vec<Fallible<String>> {
        let mut results = Vec::new();
        let dependencies = &self.manifest.merged_dependencies();
        let dependency_paths = dependencies
            .iter()
            .map(|name| self.get_dependency_path(name));

        for dependency_path in dependency_paths {
            match Manifest::for_dir(&dependency_path) {
                Ok(dependency) => {
                    for (name, _path) in dependency.bin {
                        results.push(Result::Ok(name.clone()))
                    }
                }
                Err(error) => {
                    if !error.to_string().contains("directory does not exist") {
                        results.push(Result::Err(error))
                    }
                }
            }
        }

        results
    }

    /// Convert dependency names to the path to each project.
    fn get_dependency_path(&self, name: &String) -> PathBuf {
        // ISSUE(158): Add support for Yarn Plug'n'Play.
        let mut path = PathBuf::from(&self.project_root);

        path.push("node_modules");
        path.push(name);

        path
    }

    /// Writes the specified version of Node or Yarn to the `toolchain` in package.json.
    pub fn pin(&self, distro_version: &DistroVersion) -> Fallible<()> {
        match distro_version {
            DistroVersion::Node(runtime, npm) => {
                // prevent writing the npm version if it is equal to the default version
                let default_npm = load_default_npm_version(&runtime).ok();
                let npm_str = if Some(npm.clone()) == default_npm {
                    None
                } else {
                    Some(npm.to_string())
                };

                let toolchain = serial::ToolchainSpec::new(
                    runtime.to_string(),
                    npm_str,
                    self.manifest().yarn_str().clone(),
                );
                Manifest::update_toolchain(toolchain, self.package_file())?;
            }
            DistroVersion::Yarn(version) => {
                if let Some(platform) = self.manifest().platform() {
                    let toolchain = serial::ToolchainSpec::new(
                        platform.node_runtime.to_string(),
                        platform.npm.as_ref().map(|npm| npm.to_string()),
                        Some(version.to_string()),
                    );
                    Manifest::update_toolchain(toolchain, self.package_file())?;
                } else {
                    throw!(NoPinnedNodeVersion::new());
                }
            }
            // ISSUE (#175) When we can `notion install npm` then it can be pinned in the toolchain
            DistroVersion::Npm(_) => unimplemented!("cannot pin npm in \"toolchain\""),
            DistroVersion::Package(_, _) => throw!(CannotPinPackageError::new()),
        }
        println!("Pinned {} in package.json", distro_version);
        Ok(())
    }
}

// unit tests

#[cfg(test)]
pub mod tests {
    use std::collections::{HashMap, HashSet};
    use std::ffi::OsStr;
    use std::path::PathBuf;

    use project::Project;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn gets_binary_info() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();

        let dep_bins = test_project
            .dependent_binaries()
            .expect("Could not get dependent binaries");
        let mut expected_bins = HashMap::new();
        expected_bins.insert("eslint".to_string(), "./bin/eslint.js".to_string());
        expected_bins.insert("rsvp".to_string(), "./bin/rsvp.js".to_string());
        expected_bins.insert("bin-1".to_string(), "./lib/cli.js".to_string());
        expected_bins.insert("bin-2".to_string(), "./lib/cli.js".to_string());
        assert_eq!(dep_bins, expected_bins);
    }

    #[test]
    fn gets_binary_names() {
        let project = Project::for_dir(&fixture_path("basic")).unwrap().unwrap();
        let binary_names = project.dependent_binary_names_fault_tolerant();
        let mut expected = HashSet::new();

        expected.insert("eslint".to_string());
        expected.insert("rsvp".to_string());
        expected.insert("bin-1".to_string());
        expected.insert("bin-2".to_string());

        let mut iterator = binary_names.iter();
        let mut actual = HashSet::new();

        while let Some(fallible) = iterator.next() {
            match fallible {
                Ok(binary_name) => {
                    actual.insert(binary_name.clone());
                }

                Err(error) => panic!("encountered error {:?}", error),
            }
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn local_bin_true() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // eslint, rsvp, bin-1, and bin-2 are direct dependencies
        assert!(test_project.has_direct_bin(&OsStr::new("eslint")).unwrap());
        assert!(test_project.has_direct_bin(&OsStr::new("rsvp")).unwrap());
        assert!(test_project.has_direct_bin(&OsStr::new("bin-1")).unwrap());
        assert!(test_project.has_direct_bin(&OsStr::new("bin-2")).unwrap());
    }

    #[test]
    fn local_bin_false() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // tsc and tsserver are installed, but not direct deps
        assert!(!test_project.has_direct_bin(&OsStr::new("tsc")).unwrap());
        assert!(!test_project
            .has_direct_bin(&OsStr::new("tsserver"))
            .unwrap());
    }

    #[test]
    fn maps_dependency_paths() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        let mut expected_path = PathBuf::from(project_path);

        expected_path.push("node_modules");
        expected_path.push("foo");

        assert!(test_project.get_dependency_path(&"foo".to_string()) == expected_path);
    }
}
