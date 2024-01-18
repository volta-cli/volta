use std::convert::TryFrom;
use std::fs::{read_to_string, write, File};
use std::io;
use std::path::{Path, PathBuf};

use super::empty::Empty;
use super::v1::V1;
use log::debug;
use node_semver::Version;
use tempfile::tempdir_in;
use volta_core::error::{Context, ErrorKind, Fallible, VoltaError};
use volta_core::fs::{read_dir_eager, remove_dir_if_exists, remove_file_if_exists, rename};
use volta_core::tool::load_default_npm_version;
use volta_core::toolchain::serial::Platform;
use volta_core::version::parse_version;
use volta_layout::{v1, v2};

/// Represents a V2 Volta Layout (used by Volta v0.7.3 and above)
///
/// Holds a reference to the V2 layout struct to support potential future migrations
pub struct V2 {
    pub home: v2::VoltaHome,
}

impl V2 {
    pub fn new(home: PathBuf) -> Self {
        V2 {
            home: v2::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V2 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v2::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|| ErrorKind::CreateLayoutFileError {
            file: home.layout_file().to_owned(),
        })?;

        Ok(V2 { home })
    }
}

impl TryFrom<Empty> for V2 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<V2> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v2::VoltaHome::new(old.home);
        home.create().with_context(|| ErrorKind::CreateDirError {
            dir: home.root().to_owned(),
        })?;

        V2::complete_migration(home)
    }
}

impl TryFrom<V1> for V2 {
    type Error = VoltaError;

    fn try_from(old: V1) -> Fallible<V2> {
        debug!("Migrating from V1 layout");

        let new_home = v2::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|| ErrorKind::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Perform the core of the migration
        clear_default_npm(old.home.default_platform_file())?;
        shift_node_images(&old.home, &new_home)?;

        // Complete the migration, writing the V2 layout file
        let layout = V2::complete_migration(new_home)?;

        // Remove the V1 layout file, since we're now on V2 (do this after writing the V2 so that we know the migration succeeded)
        let old_layout_file = old.home.layout_file();
        remove_file_if_exists(old_layout_file)?;
        Ok(layout)
    }
}

/// Clear npm from the default `platform.json` file if it is set to the same value as that bundled with Node
///
/// This will ensure that we don't treat the default npm from a prior version of Volta as a "custom" npm that
/// the user explicitly requested
fn clear_default_npm(platform_file: &Path) -> Fallible<()> {
    let platform_json = match read_to_string(platform_file) {
        Ok(json) => json,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(());
            } else {
                return Err(VoltaError::from_source(
                    error,
                    ErrorKind::ReadPlatformError {
                        file: platform_file.to_path_buf(),
                    },
                ));
            }
        }
    };
    let mut existing_platform = Platform::try_from(platform_json)?;

    if let Some(ref mut node_version) = &mut existing_platform.node {
        if let Some(npm) = &node_version.npm {
            if let Ok(default_npm) = load_default_npm_version(&node_version.runtime) {
                if *npm == default_npm {
                    node_version.npm = None;
                    write(platform_file, existing_platform.into_json()?).with_context(|| {
                        ErrorKind::WritePlatformError {
                            file: platform_file.to_owned(),
                        }
                    })?;
                }
            }
        }
    }

    Ok(())
}

/// Move all Node images up one directory, removing the default npm version directory
///
/// In the V1 layout, we kept all node images in /<node_version>/<npm_version>/, however we will be
/// storing custom npm versions in a separate image directory, so there is no need to maintain the
/// bundled npm version in the file structure any more. This also will make it slightly easier to access
/// the Node image, as we no longer will need to look up the bundled npm version every time.
fn shift_node_images(old_home: &v1::VoltaHome, new_home: &v2::VoltaHome) -> Fallible<()> {
    let temp_dir =
        tempdir_in(new_home.tmp_dir()).with_context(|| ErrorKind::CreateTempDirError {
            in_dir: new_home.tmp_dir().to_owned(),
        })?;
    let node_installs = read_dir_eager(old_home.node_image_root_dir())
        .with_context(|| ErrorKind::ReadDirError {
            dir: old_home.node_image_root_dir().to_owned(),
        })?
        .filter_map(|(entry, metadata)| {
            if metadata.is_dir() {
                parse_version(entry.file_name().to_string_lossy()).ok()
            } else {
                None
            }
        });

    for node_version in node_installs {
        remove_npm_version_from_node_image_dir(old_home, new_home, node_version, temp_dir.path())?;
    }

    Ok(())
}

/// Move a single node image up a directory, if it currently has the npm version in its path
fn remove_npm_version_from_node_image_dir(
    old_home: &v1::VoltaHome,
    new_home: &v2::VoltaHome,
    node_version: Version,
    temp_dir: &Path,
) -> Fallible<()> {
    let node_string = node_version.to_string();
    let npm_version = load_default_npm_version(&node_version)?;
    let old_install = old_home.node_image_dir(&node_string, &npm_version.to_string());

    if old_install.exists() {
        let temp_image = temp_dir.join(&node_string);
        let new_install = new_home.node_image_dir(&node_string);
        rename(&old_install, &temp_image).with_context(|| ErrorKind::SetupToolImageError {
            tool: "Node".into(),
            version: node_string.clone(),
            dir: temp_image.clone(),
        })?;
        remove_dir_if_exists(&new_install)?;
        rename(&temp_image, &new_install).with_context(|| ErrorKind::SetupToolImageError {
            tool: "Node".into(),
            version: node_string,
            dir: temp_image,
        })?;
    }
    Ok(())
}
