//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use lazycell::LazyCell;
use semver::Version;

use crate::distro::node::{load_default_npm_version, NodeVersion};
use crate::distro::package::BinConfig;
use crate::error::ErrorDetails;
use crate::manifest::{serial, Manifest};
use crate::path;
use crate::platform::PlatformSpec;
use volta_fail::{throw, Fallible, ResultExt};

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
        match path::find_project_dir(base_dir) {
            Some(dir) => Ok(Some(Rc::new(Project {
                manifest: Manifest::for_dir(&dir)?,
                project_root: PathBuf::from(dir),
            }))),
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
            let config_path = path::user_tool_bin_config(name)?;
            if config_path.exists() {
                let config = BinConfig::from_file(config_path)?;
                return Ok(self.has_direct_dependency(&config.package));
            }
        }
        Ok(false)
    }

    fn has_direct_dependency(&self, dependency: &str) -> bool {
        self.manifest.dependencies.contains_key(dependency)
            || self.manifest.dev_dependencies.contains_key(dependency)
    }

    /// Writes the specified version of Node to the `volta.node` key in package.json.
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
        Ok(())
    }

    /// Writes the specified version of Yarn to the `volta.yarn` key in package.json.
    pub fn pin_yarn(&self, yarn_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest().platform() {
            let toolchain = serial::ToolchainSpec::new(
                platform.node_runtime.to_string(),
                platform.npm.as_ref().map(|npm| npm.to_string()),
                Some(yarn_version.to_string()),
            );
            Manifest::update_toolchain(toolchain, self.package_file())?;
        } else {
            throw!(ErrorDetails::NoPinnedNodeVersion);
        }
        Ok(())
    }

    /// Writes the specified version of Npm to the `volta.npm` key in package.json.
    pub fn pin_npm(&self, npm_version: &Version) -> Fallible<()> {
        if let Some(platform) = self.manifest().platform() {
            let toolchain = serial::ToolchainSpec::new(
                platform.node_runtime.to_string(),
                Some(npm_version.to_string()),
                self.manifest().yarn_str().clone(),
            );
            Manifest::update_toolchain(toolchain, self.package_file())?;
        } else {
            throw!(ErrorDetails::NoPinnedNodeVersion);
        }
        Ok(())
    }
}

// unit tests

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use crate::project::Project;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn direct_dependency_true() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // eslint, rsvp, bin-1, and bin-2 are direct dependencies
        assert!(test_project.has_direct_dependency("eslint"));
        assert!(test_project.has_direct_dependency("rsvp"));
        assert!(test_project.has_direct_dependency("@namespace/some-dep"));
        assert!(test_project.has_direct_dependency("@namespaced/something-else"));
    }

    #[test]
    fn direct_dependency_false() {
        let project_path = fixture_path("basic");
        let test_project = Project::for_dir(&project_path).unwrap().unwrap();
        // tsc and tsserver are installed, but not direct deps
        assert!(!test_project.has_direct_dependency("typescript"));
    }
}
