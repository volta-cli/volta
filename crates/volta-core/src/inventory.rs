//! Provides types for working with Volta's _inventory_, the local repository
//! of available tool versions.

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::read_dir_eager;
use crate::layout::volta_home;
use crate::tool::PackageConfig;
use crate::version::parse_version;
use log::debug;
use semver::Version;
use walkdir::WalkDir;

/// Checks if a given Node version image is available on the local machine
pub fn node_available(version: &Version) -> Fallible<bool> {
    volta_home().map(|home| {
        home.node_image_root_dir()
            .join(version.to_string())
            .exists()
    })
}

/// Collects a set of all Node versions fetched on the local machine
pub fn node_versions() -> Fallible<BTreeSet<Version>> {
    volta_home().and_then(|home| read_versions(home.node_image_root_dir()))
}

/// Checks if a given npm version image is available on the local machine
pub fn npm_available(version: &Version) -> Fallible<bool> {
    volta_home().map(|home| home.npm_image_dir(&version.to_string()).exists())
}

/// Collects a set of all npm versions fetched on the local machine
pub fn npm_versions() -> Fallible<BTreeSet<Version>> {
    volta_home().and_then(|home| read_versions(home.npm_image_root_dir()))
}

/// Checks if a given Yarn version image is available on the local machine
pub fn yarn_available(version: &Version) -> Fallible<bool> {
    volta_home().map(|home| home.yarn_image_dir(&version.to_string()).exists())
}

/// Collects a set of all Yarn versions fetched on the local machine
pub fn yarn_versions() -> Fallible<BTreeSet<Version>> {
    volta_home().and_then(|home| read_versions(home.yarn_image_root_dir()))
}

/// Checks if a given package version image is available on the local machine
#[cfg(not(feature = "package-global"))]
pub fn package_available(name: &str, version: &Version) -> Fallible<bool> {
    volta_home().map(|home| home.package_image_dir(name, &version.to_string()).exists())
}

/// Collects a set of all Package Configs on the local machine
pub fn package_configs() -> Fallible<BTreeSet<PackageConfig>> {
    let package_dir = volta_home()?.default_package_dir();

    WalkDir::new(&package_dir)
        .max_depth(2)
        .into_iter()
        // Ignore any items which didn't resolve as `DirEntry` correctly.
        // There is no point trying to do anything with those, and no error
        // we can report to the user in any case. Log the failure in the
        // debug output, though
        .filter_map(|entry| match entry {
            Ok(dir_entry) => {
                // Ignore directory entries.
                if dir_entry.file_type().is_file() {
                    Some(dir_entry.into_path())
                } else {
                    None
                }
            }
            Err(e) => {
                debug!("{}", e);
                None
            }
        })
        .map(|file_path| PackageConfig::from_file(&file_path))
        .collect::<Fallible<BTreeSet<PackageConfig>>>()
}

/// Reads the contents of a directory and returns the set of all versions found
/// in the directory's listing by parsing the directory names as semantic versions
fn read_versions(dir: &Path) -> Fallible<BTreeSet<Version>> {
    let contents = read_dir_eager(dir).with_context(|| ErrorKind::ReadDirError {
        dir: dir.to_owned(),
    })?;

    Ok(contents
        .filter(|(_, metadata)| metadata.is_dir())
        .filter_map(|(entry, _)| parse_version(entry.file_name().to_string_lossy()).ok())
        .collect())
}
