//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use lazycell::LazyCell;
use semver::Version;

use crate::error::ErrorDetails;
use crate::layout::volta_home;
use crate::manifest::Manifest;
use crate::platform::PlatformSpec;
use crate::tool::{load_default_npm_version, BinConfig, NodeVersion};
use log::debug;
use volta_fail::{Fallible, ResultExt};

/// A lazily loaded Project
pub struct LazyProject {
    project: LazyCell<Option<Project>>,
}

impl LazyProject {
    pub fn init() -> Self {
        LazyProject {
            project: LazyCell::new(),
        }
    }

    pub fn get(&self) -> Fallible<Option<&Project>> {
        let project = self.project.try_borrow_with(Project::for_current_dir)?;
        Ok(project.as_ref())
    }

    pub fn get_mut(&mut self) -> Fallible<Option<&mut Project>> {
        let project = self.project.try_borrow_mut_with(Project::for_current_dir)?;
        Ok(project.as_mut())
    }
}

/// A Node project tree in the filesystem.
pub struct Project {
    manifest: Manifest,
    project_root: PathBuf,
}

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

impl Project {
    /// Returns the Node project containing the current working directory,
    /// if any.
    fn for_current_dir() -> Fallible<Option<Project>> {
        let current_dir: &Path =
            &env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?;
        Self::for_dir(&current_dir)
    }

    /// Starts at `base_dir` and walks up the directory tree until a package.json file is found
    pub(crate) fn find_dir(base_dir: &Path) -> Option<&Path> {
        let mut dir = base_dir;
        while !is_project_root(dir) {
            dir = match dir.parent() {
                Some(parent) => parent,
                None => {
                    return None;
                }
            }
        }

        Some(dir)
    }

    /// Returns the Node project for the input directory, if any.
    fn for_dir(base_dir: &Path) -> Fallible<Option<Project>> {
        match Self::find_dir(base_dir) {
            Some(dir) => {
                debug!("Found project manifest in '{}'", dir.display());
                Ok(Some(Project {
                    manifest: Manifest::for_dir(&dir)?,
                    project_root: PathBuf::from(dir),
                }))
            }
            None => Ok(None),
        }
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
        if let Some(name) = bin_name.to_str() {
            let config_path = volta_home()?.user_tool_bin_config(name);
            if config_path.exists() {
                let config = BinConfig::from_file(config_path)?;
                return Ok(self.has_direct_dependency(&config.package));
            }
        }
        Ok(false)
    }

    /// Returns a matching config if the bin exists at the specified version in
    /// the project.
    pub fn matching_bin(&self, bin_name: &OsStr, version: &Version) -> Fallible<Option<BinConfig>> {
        let home = volta_home()?;
        let config_path = bin_name
            .to_str()
            .map(|name| home.user_tool_bin_config(name));

        let bin_config = config_path.map(BinConfig::from_file).transpose()?;

        let matching_config = bin_config.and_then(|config| {
            if self.has_direct_dependency(&config.package) && &config.version == version {
                Some(config)
            } else {
                None
            }
        });

        Ok(matching_config)
    }

    fn has_direct_dependency(&self, dependency: &str) -> bool {
        self.manifest.dependencies.contains_key(dependency)
            || self.manifest.dev_dependencies.contains_key(dependency)
    }

    pub fn has_dependency(&self, dependency: &str, version: &Version) -> bool {
        let has_dep = |deps: &HashMap<String, String>| {
            deps.get(dependency)
                .and_then(|v| Version::from_str(v).ok())
                .map(|v| &v == version)
        };

        has_dep(&self.manifest.dependencies)
            .or_else(|| has_dep(&self.manifest.dev_dependencies))
            .unwrap_or(false)
    }

    /// Writes the specified version of Node to the `volta.node` key in package.json.
    pub fn pin_node(&mut self, node_version: &NodeVersion) -> Fallible<()> {
        // prevent writing the npm version if it is equal to the default version

        let npm = load_default_npm_version(&node_version.runtime)
            .ok()
            .and_then(|default| {
                if node_version.npm == default {
                    debug!("Not writing 'npm' key since the version matches the Node default");
                    None
                } else {
                    Some(node_version.npm.clone())
                }
            });

        let updated_platform = PlatformSpec {
            node_runtime: node_version.runtime.clone(),
            npm,
            yarn: self.manifest.yarn(),
        };

        self.manifest.update_platform(updated_platform);
        self.manifest.write(self.package_file())
    }

    /// Writes the specified version of Yarn to the `volta.yarn` key in package.json.
    pub fn pin_yarn(&mut self, yarn_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest.platform() {
            let updated_platform = PlatformSpec {
                node_runtime: platform.node_runtime.clone(),
                npm: platform.npm.clone(),
                yarn: Some(yarn_version.clone()),
            };

            self.manifest.update_platform(updated_platform);
            self.manifest.write(self.package_file())
        } else {
            Err(ErrorDetails::NoPinnedNodeVersion.into())
        }
    }

    /// Writes the specified version of Npm to the `volta.npm` key in package.json.
    pub fn pin_npm(&mut self, npm_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest.platform() {
            let updated_platform = PlatformSpec {
                node_runtime: platform.node_runtime.clone(),
                npm: Some(npm_version.clone()),
                yarn: self.manifest.yarn(),
            };

            self.manifest.update_platform(updated_platform);
            self.manifest.write(self.package_file())
        } else {
            Err(ErrorDetails::NoPinnedNodeVersion.into())
        }
    }
}

// unit tests

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use crate::project::Project;

    fn fixture_path(fixture_dirs: &[&str]) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");

        for fixture_dir in fixture_dirs.iter() {
            cargo_manifest_dir.push(fixture_dir);
        }

        cargo_manifest_dir
    }

    #[test]
    fn direct_dependency_true() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // eslint, rsvp, bin-1, and bin-2 are direct dependencies
        assert!(test_project.has_direct_dependency("eslint"));
        assert!(test_project.has_direct_dependency("rsvp"));
        assert!(test_project.has_direct_dependency("@namespace/some-dep"));
        assert!(test_project.has_direct_dependency("@namespaced/something-else"));
    }

    #[test]
    fn direct_dependency_false() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // tsc and tsserver are installed, but not direct deps
        assert!(!test_project.has_direct_dependency("typescript"));
    }

    #[test]
    fn test_project_find_dir_direct() {
        let base_dir = fixture_path(&["basic"]);
        let project_dir = Project::find_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, base_dir);
    }

    #[test]
    fn test_project_find_dir_ancestor() {
        let base_dir = fixture_path(&["basic", "subdir"]);
        let project_dir = Project::find_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }

    #[test]
    fn test_project_find_dir_dependency() {
        let base_dir = fixture_path(&["basic", "node_modules", "eslint"]);
        let project_dir = Project::find_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }
}
