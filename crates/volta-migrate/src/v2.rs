use std::convert::TryFrom;
use std::fs::{read_to_string, remove_file, rename, write, File};
use std::path::PathBuf;

use super::empty::Empty;
use super::v1::V1;
use log::debug;
use tempfile::tempdir_in;
use volta_core::error::ErrorDetails;
use volta_core::fs::{ensure_dir_does_not_exist, read_dir_eager};
use volta_core::tool::load_default_npm_version;
use volta_core::toolchain::serial::Platform;
use volta_core::version::parse_version;
use volta_fail::{Fallible, ResultExt, VoltaError};
use volta_layout::v2;

/// Represents a V2 Volta Layout (from v0.7.3)
///
/// Holds a reference to the V1 layout struct to support potential future migrations
pub struct V2 {
    pub home: v2::VoltaHome,
}

impl V2 {
    pub fn new(home: PathBuf) -> Self {
        V2 {
            home: v2::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V1 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v2::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|_| ErrorDetails::CreateLayoutFileError {
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
        home.create()
            .with_context(|_| ErrorDetails::CreateDirError {
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
            .with_context(|_| ErrorDetails::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Check the default platform file `platform.json`
        // If it contains an npm version that matches the default, update it to have None instead
        // This will ensure that we don't treat the default npm from a prior version of Volta
        // as a "custom" npm that the user explicitly requested
        let platform_file = old.home.default_platform_file();
        if platform_file.exists() {
            let platform_json = read_to_string(platform_file).with_context(|_| {
                ErrorDetails::ReadPlatformError {
                    file: platform_file.to_owned(),
                }
            })?;
            let mut existing_platform = Platform::from_json(platform_json)?;

            if let Some(ref mut node_version) = &mut existing_platform.node {
                if let Some(npm) = &node_version.npm {
                    if *npm == load_default_npm_version(&node_version.runtime)? {
                        node_version.npm = None;
                        write(platform_file, existing_platform.into_json()?).with_context(
                            |_| ErrorDetails::WritePlatformError {
                                file: platform_file.to_owned(),
                            },
                        )?;
                    }
                }
            }
        }

        // Move node_image_dir Up one directory (V1 -> V2)
        let temp_dir =
            tempdir_in(new_home.tmp_dir()).with_context(|_| ErrorDetails::CreateTempDirError {
                in_dir: new_home.tmp_dir().to_owned(),
            })?;
        let node_installs = read_dir_eager(old.home.node_image_root_dir())
            .with_context(|_| ErrorDetails::ReadDirError {
                dir: old.home.node_image_root_dir().to_owned(),
            })?
            .filter_map(|(entry, metadata)| {
                if metadata.is_dir() {
                    parse_version(entry.file_name().to_string_lossy()).ok()
                } else {
                    None
                }
            });

        for node_version in node_installs {
            let npm_version = load_default_npm_version(&node_version)?;
            let old_install = old
                .home
                .node_image_dir(&node_version.to_string(), &npm_version.to_string());

            if old_install.exists() {
                let temp_image = temp_dir.path().join(node_version.to_string());
                let new_install = new_home.node_image_dir(&node_version.to_string());
                rename(&old_install, &temp_image).with_context(|_| {
                    ErrorDetails::SetupToolImageError {
                        tool: "Node".to_string(),
                        version: node_version.to_string(),
                        dir: temp_image.clone(),
                    }
                })?;
                ensure_dir_does_not_exist(&new_install)?;
                rename(&temp_image, &new_install).with_context(|_| {
                    ErrorDetails::SetupToolImageError {
                        tool: "Node".to_string(),
                        version: node_version.to_string(),
                        dir: new_install.clone(),
                    }
                })?;
            }
        }

        // Complete the migration, writing the V2 layout file
        let layout = V2::complete_migration(new_home)?;

        // Remove the V1 layout file, since we're now on V2 (do this after writing the V2 so that we know the migration succeeded)
        let old_layout_file = old.home.layout_file();
        if old_layout_file.exists() {
            remove_file(old_layout_file).with_context(|_| ErrorDetails::DeleteFileError {
                file: old_layout_file.to_owned(),
            })?;
        }

        Ok(layout)
    }
}
