//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use lazycell::LazyCell;

use manifest::Manifest;
use notion_fail::{Fallible, NotionError, NotionFail, ResultExt};
use package_info::PackageInfo;

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

#[derive(Fail, Debug)]
#[fail(display = "Could not read dependent package info: {}", error)]
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

impl NotionFail for DepPackageReadError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> i32 {
        4
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

        let manifest = match Manifest::for_dir(&dir)? {
            Some(manifest) => manifest,
            None => {
                return Ok(None);
            }
        };

        Ok(Some(Project {
            manifest: manifest,
            project_root: PathBuf::from(dir),
            dependent_bins: LazyDependentBins::new(),
        }))
    }

    /// Returns the project manifest (`package.json`) for this project.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Returns the path to the local binary directory for this project.
    pub fn local_bin_dir(&self) -> PathBuf {
        let sub_dir: PathBuf = ["node_modules", ".bin"].iter().collect();
        self.project_root.join(sub_dir)
    }

    /// Returns true if the input binary name is a direct dependency of the input project
    pub fn has_local_bin(&self, bin_name: &OsStr) -> Fallible<bool> {
        let dep_bins = self.dependent_bins.get(&self)?;
        if let Some(bin_name_str) = bin_name.to_str() {
            if dep_bins.contains_key(bin_name_str) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Gets the names of all the direct dependencies of the current project
    fn all_dependencies(&self) -> Fallible<Option<HashSet<String>>> {
        if let Some(manifest) = Manifest::for_dir(&self.project_root)? {
            let mut dependencies = HashSet::new();
            for (name, _version) in manifest.dependencies.iter() {
                dependencies.insert(name.clone());
            }
            for (name, _version) in manifest.dev_dependencies.iter() {
                dependencies.insert(name.clone());
            }
            return Ok(Some(dependencies));
        }
        Ok(None)
    }

    /// Returns a mapping of the names to paths for all the binaries installed
    /// by direct dependencies of the current project.
    fn dependent_binaries(&self) -> Fallible<HashMap<String, String>> {
        let mut dependent_bins = HashMap::new();
        if let Some(all_deps) = self.all_dependencies()? {
            // convert dependency names to the path to each project
            let all_dep_paths = all_deps
                .iter()
                .map(|dep_name| {
                    let mut path_to_pkg = PathBuf::from(&self.project_root);
                    path_to_pkg.push("node_modules");
                    path_to_pkg.push(dep_name);
                    path_to_pkg
                })
                .collect::<HashSet<PathBuf>>();

            // use those project paths to get the "bin" info for each project
            for pkg_path in all_dep_paths.iter() {
                let pkg_info =
                    PackageInfo::for_dir(&pkg_path).with_context(DepPackageReadError::from_error)?;
                let bin_map = pkg_info.bin;
                for (name, path) in bin_map.iter() {
                    dependent_bins.insert(name.clone(), path.clone());
                }
            }
        }
        Ok(dependent_bins)
    }
}

// unit tests

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
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
    fn gets_all_dependencies() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();

        let all_deps = match test_project.all_dependencies() {
            Ok(deps) => deps,
            _ => panic!(
                "Error: Could not get dependencies for project {:?}",
                project_path
            ),
        };
        let mut expected_deps = HashSet::new();
        expected_deps.insert("@namespace/some-dep".to_string());
        expected_deps.insert("rsvp".to_string());
        expected_deps.insert("@namespaced/something-else".to_string());
        expected_deps.insert("eslint".to_string());
        assert_eq!(all_deps, Some(expected_deps));
    }

    #[test]
    fn gets_binary_info() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();

        let dep_bins = match test_project.dependent_binaries() {
            Ok(bin_map) => bin_map,
            _ => panic!(
                "Error: Could not get dependent binaries for project {:?}",
                project_path
            ),
        };
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
        assert!(test_project.has_local_bin(&OsStr::new("eslint")).unwrap());
        assert!(test_project.has_local_bin(&OsStr::new("rsvp")).unwrap());
        assert!(test_project.has_local_bin(&OsStr::new("bin-1")).unwrap());
        assert!(test_project.has_local_bin(&OsStr::new("bin-2")).unwrap());
    }

    #[test]
    fn local_bin_false() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // tsc and tsserver are installed, but not direct deps
        assert!(!test_project.has_local_bin(&OsStr::new("tsc")).unwrap());
        assert!(!test_project.has_local_bin(&OsStr::new("tsserver")).unwrap());
    }
}
