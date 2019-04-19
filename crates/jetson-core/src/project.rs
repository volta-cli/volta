//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use lazycell::LazyCell;
use semver::Version;

use crate::distro::node::{load_default_npm_version, NodeVersion};
use crate::error::ErrorDetails;
use crate::manifest::{serial, Manifest};
use crate::platform::PlatformSpec;
use jetson_fail::{throw, Fallible, ResultExt};

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
        self.bins.try_borrow_with(|| project.dependent_binaries())
    }
}

/// A lazily loaded Project
pub struct LazyProject {
    project: LazyCell<Option<Rc<Project>>>,
}

impl LazyProject {
    pub fn new() -> Self {
        LazyProject {
            project: LazyCell::new(),
        }
    }

    pub fn get(&self) -> Fallible<Option<Rc<Project>>> {
        let project = self
            .project
            .try_borrow_with(|| Project::for_current_dir())?;
        Ok(project.clone())
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
    fn for_current_dir() -> Fallible<Option<Rc<Project>>> {
        let current_dir: &Path =
            &env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?;
        Self::for_dir(&current_dir)
    }

    /// Returns the Node project for the input directory, if any.
    fn for_dir(base_dir: &Path) -> Fallible<Option<Rc<Project>>> {
        let mut dir = base_dir.clone();
        while !is_project_root(dir) {
            dir = match dir.parent() {
                Some(parent) => parent,
                None => {
                    return Ok(None);
                }
            }
        }

        Ok(Some(Rc::new(Project {
            manifest: Manifest::for_dir(&dir)?,
            project_root: PathBuf::from(dir),
            dependent_bins: LazyDependentBins::new(),
        })))
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

    /// Returns a mapping of the names to paths for all the binaries installed
    /// by direct dependencies of the current project.
    fn dependent_binaries(&self) -> Fallible<HashMap<String, String>> {
        let mut dependent_bins = HashMap::new();
        let all_deps = Manifest::for_dir(&self.project_root)?.merged_dependencies();
        let all_dep_paths = all_deps.iter().map(|name| self.get_dependency_path(name));

        // use those project paths to get the "bin" info for each project
        for pkg_path in all_dep_paths {
            let pkg_info =
                Manifest::for_dir(&pkg_path).with_context(|_| ErrorDetails::DepPackageReadError)?;
            let bin_map = pkg_info.bin;
            for (name, path) in bin_map.iter() {
                dependent_bins.insert(name.clone(), path.clone());
            }
        }
        Ok(dependent_bins)
    }

    /// Convert dependency names to the path to each project.
    fn get_dependency_path(&self, name: &String) -> PathBuf {
        // ISSUE(158): Add support for Yarn Plug'n'Play.
        let mut path = PathBuf::from(&self.project_root);

        path.push("node_modules");
        path.push(name);

        path
    }

    /// Writes the specified version of Node to the `toolchain.node` key in package.json.
    pub fn pin_node(&self, node_version: &NodeVersion) -> Fallible<()> {
        // prevent writing the npm version if it is equal to the default version

        let npm_str = load_default_npm_version(&node_version.runtime)
            .ok()
            .and_then(|default| {
                if node_version.npm == default {
                    None
                } else {
                    Some(node_version.npm.to_string())
                }
            });

        let toolchain = serial::ToolchainSpec::new(
            node_version.runtime.to_string(),
            npm_str,
            self.manifest().yarn_str().clone(),
        );
        Manifest::update_toolchain(toolchain, self.package_file())?;
        println!(
            "Pinned node version {} (with npm {}) in package.json",
            node_version.runtime, node_version.npm
        );
        Ok(())
    }

    /// Writes the specified version of Yarn to the `toolchain.yarn` key in package.json.
    pub fn pin_yarn(&self, yarn_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest().platform() {
            let toolchain = serial::ToolchainSpec::new(
                platform.node_runtime.to_string(),
                platform.npm.as_ref().map(|npm| npm.to_string()),
                Some(yarn_version.to_string()),
            );
            Manifest::update_toolchain(toolchain, self.package_file())?;
            println!("Pinned yarn version {} in package.json", yarn_version);
        } else {
            throw!(ErrorDetails::NoPinnedNodeVersion);
        }
        Ok(())
    }

    /// Writes the specified version of Npm to the `toolchain.npm` key in package.json.
    pub fn pin_npm(&self, npm_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest().platform() {
            let toolchain = serial::ToolchainSpec::new(
                platform.node_runtime.to_string(),
                Some(npm_version.to_string()),
                self.manifest().yarn_str().clone(),
            );
            Manifest::update_toolchain(toolchain, self.package_file())?;
            println!("Pinned npm version {} in package.json", npm_version);
        } else {
            throw!(ErrorDetails::NoPinnedNodeVersion);
        }
        Ok(())
    }
}

// unit tests

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    use crate::project::Project;

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
