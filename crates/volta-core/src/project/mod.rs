//! Provides the `Project` type, which represents a Node project tree in
//! the filesystem.

use std::convert::{TryFrom, TryInto};
use std::env;
use std::ffi::OsStr;
use std::iter::once;
use std::path::{Path, PathBuf};

use lazycell::LazyCell;
use semver::Version;

use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::layout::volta_home;
use crate::platform::PlatformSpec;
use crate::tool::BinConfig;
use chain_map::ChainMap;
use indexmap::IndexSet;

mod serial;

use serial::{update_manifest_node, update_manifest_npm, update_manifest_yarn, Manifest};

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

/// A Node project workspace in the filesystem
pub struct Project {
    manifest_file: PathBuf,
    extensions: IndexSet<PathBuf>,
    dependencies: ChainMap<String, String>,
    platform: Option<PlatformSpec>,
}

impl Project {
    /// Creates an optional Project instance from the current directory
    fn for_current_dir() -> Fallible<Option<Self>> {
        let current_dir = env::current_dir().with_context(|| ErrorKind::CurrentDirError)?;
        Self::for_dir(current_dir)
    }

    /// Creates an optional Project instance from the specified directory
    ///
    /// Will search ancestors to find a `package.json` and use that as the root of the project
    fn for_dir(base_dir: PathBuf) -> Fallible<Option<Self>> {
        match find_closest_root(base_dir) {
            Some(mut project) => {
                project.push("package.json");
                Self::from_file(project).map(Some)
            }
            None => Ok(None),
        }
    }

    /// Creates a Project instance from the given package manifest file (`package.json`)
    fn from_file(manifest_file: PathBuf) -> Fallible<Self> {
        let manifest = Manifest::from_file(&manifest_file)?;
        let mut dependencies: ChainMap<String, String> = manifest.dependency_maps.collect();
        let mut extensions = IndexSet::new();
        let mut platform = manifest.platform;
        let mut extends = manifest.extends;

        // Iterate the `volta.extends` chain, parsing each file in turn
        while let Some(path) = extends {
            // Detect cycles to prevent infinite looping
            if path == manifest_file || extensions.contains(&path) {
                return Err(ErrorKind::ExtensionCycleError { file: path }.into());
            }

            let manifest = Manifest::from_file(&path)?;
            extensions.insert(path);

            for map in manifest.dependency_maps {
                dependencies.push_map(map);
            }

            platform = match (platform, manifest.platform) {
                (Some(base), Some(ext)) => Some(base.merge(ext)),
                (Some(plat), None) | (None, Some(plat)) => Some(plat),
                (None, None) => None,
            };

            extends = manifest.extends;
        }

        let platform = platform.map(TryInto::try_into).transpose()?;

        Ok(Project {
            manifest_file,
            dependencies,
            extensions,
            platform,
        })
    }

    /// Returns a reference to the manifest file for the current project
    pub fn manifest_file(&self) -> &Path {
        &self.manifest_file
    }

    /// Returns an iterator of paths to all of the workspace roots
    pub fn workspace_roots(&self) -> impl Iterator<Item = &Path> {
        // Invariant: self.manifest_file and self.extensions will only contain paths to files
        // So they will _always_ have a parent
        once(&self.manifest_file)
            .chain(self.extensions.iter())
            .map(|file| file.parent().unwrap())
    }

    /// Returns a reference to the Project's `PlatformSpec`, if available
    pub fn platform(&self) -> Option<&PlatformSpec> {
        self.platform.as_ref()
    }

    /// Returns true if the project dependency map contains the specified dependency
    pub fn has_direct_dependency(&self, dependency: &str) -> bool {
        self.dependencies.contains_key(dependency)
    }

    /// Returns true if the input binary name is a direct dependency of the input project
    pub fn has_direct_bin(&self, bin_name: &OsStr) -> Fallible<bool> {
        if let Some(name) = bin_name.to_str() {
            let config_path = volta_home()?.default_tool_bin_config(name);
            if config_path.exists() {
                let config = BinConfig::from_file(config_path)?;
                return Ok(self.has_direct_dependency(&config.package));
            }
        }
        Ok(false)
    }

    /// Searches the project roots to find the path to a project-local binary file
    pub fn find_bin(&self, bin_name: &OsStr) -> Option<PathBuf> {
        self.workspace_roots().find_map(|root| {
            let mut bin_path = root.join("node_modules");
            bin_path.push(".bin");
            bin_path.push(bin_name);

            if bin_path.is_file() {
                Some(bin_path)
            } else {
                None
            }
        })
    }

    /// Pins the Node version in this project's manifest file
    pub fn pin_node(&mut self, version: &Version) -> Fallible<()> {
        if let Some(platform) = self.platform.as_mut() {
            platform.node = version.clone();
        } else {
            self.platform = Some(PlatformSpec {
                node: version.clone(),
                npm: None,
                yarn: None,
            });
        }

        update_manifest_node(&self.manifest_file, Some(version))
    }

    /// Pins the npm version in this project's manifest file
    pub fn pin_npm(&mut self, version: Option<&Version>) -> Fallible<()> {
        if let Some(platform) = self.platform.as_mut() {
            platform.npm = version.cloned();

            update_manifest_npm(&self.manifest_file, version)
        } else {
            Err(ErrorKind::NoPinnedNodeVersion { tool: "npm".into() }.into())
        }
    }

    /// Pins the Yarn version in this project's manifest file
    pub fn pin_yarn(&mut self, version: Option<&Version>) -> Fallible<()> {
        if let Some(platform) = self.platform.as_mut() {
            platform.yarn = version.cloned();

            update_manifest_yarn(&self.manifest_file, version)
        } else {
            Err(ErrorKind::NoPinnedNodeVersion {
                tool: "Yarn".into(),
            }
            .into())
        }
    }

    // TODO: Remove these once the changes to `volta list` are merged
    pub fn matching_bin(&self, _bin: &OsStr, _version: &Version) -> Fallible<Option<BinConfig>> {
        Ok(None)
    }

    pub fn package_file(&self) -> PathBuf {
        self.manifest_file.clone()
    }

    pub fn has_dependency(&self, _dep: &str, _version: &Version) -> bool {
        false
    }
}

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").exists()
}

fn is_node_modules(dir: &Path) -> bool {
    dir.file_name().map_or(false, |tail| tail == "node_modules")
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, is_node_modules)
}

fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

/// Starts at `base_dir` and walks up the directory tree until a package.json file is found
pub(crate) fn find_closest_root(mut dir: PathBuf) -> Option<PathBuf> {
    while !is_project_root(&dir) {
        if !dir.pop() {
            return None;
        }
    }

    Some(dir)
}

struct PartialPlatform {
    node: Option<Version>,
    npm: Option<Version>,
    yarn: Option<Version>,
}

impl PartialPlatform {
    fn merge(self, other: PartialPlatform) -> PartialPlatform {
        PartialPlatform {
            node: self.node.or(other.node),
            npm: self.npm.or(other.npm),
            yarn: self.yarn.or(other.yarn),
        }
    }
}

impl TryFrom<PartialPlatform> for PlatformSpec {
    type Error = VoltaError;

    fn try_from(partial: PartialPlatform) -> Fallible<PlatformSpec> {
        let node = partial.node.ok_or(ErrorKind::NoProjectNodeInManifest)?;

        Ok(PlatformSpec {
            node,
            npm: partial.npm,
            yarn: partial.yarn,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use super::*;

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
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        // eslint, rsvp, bin-1, and bin-2 are direct dependencies
        assert!(test_project.has_direct_dependency("eslint"));
        assert!(test_project.has_direct_dependency("rsvp"));
        assert!(test_project.has_direct_dependency("@namespace/some-dep"));
        assert!(test_project.has_direct_dependency("@namespaced/something-else"));
    }

    #[test]
    fn direct_dependency_false() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        // tsc and tsserver are installed, but not direct deps
        assert!(!test_project.has_direct_dependency("typescript"));
    }

    #[test]
    fn test_find_closest_root_direct() {
        let base_dir = fixture_path(&["basic"]);
        let project_dir =
            find_closest_root(base_dir.clone()).expect("Failed to find project directory");

        assert_eq!(project_dir, base_dir);
    }

    #[test]
    fn test_find_closest_root_ancestor() {
        let base_dir = fixture_path(&["basic", "subdir"]);
        let project_dir = find_closest_root(base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }

    #[test]
    fn test_find_closest_root_dependency() {
        let base_dir = fixture_path(&["basic", "node_modules", "eslint"]);
        let project_dir = find_closest_root(base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }
}