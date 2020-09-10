use std::convert::TryFrom;
use std::fs::File;
use std::path::PathBuf;

use crate::empty::Empty;
use crate::v2::V2;
use log::debug;
use volta_core::error::{Context, ErrorKind, Fallible, VoltaError};
use volta_core::fs::remove_file_if_exists;
use volta_core::platform::PlatformSpec;
use volta_core::session::Session;
use volta_core::tool::Package;
use volta_core::version::VersionSpec;
use volta_layout::{v2, v3};
use walkdir::WalkDir;

mod config;

use config::LegacyPackageConfig;

/// Represents a V3 Volta layout (used by Volta v0.9.0 and above)
///
/// Holds a reference to the V3 layout struct to support future migrations
pub struct V3 {
    pub home: v3::VoltaHome,
}

impl V3 {
    pub fn new(home: PathBuf) -> Self {
        V3 {
            home: v3::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V2 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v3::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|| ErrorKind::CreateLayoutFileError {
            file: home.layout_file().to_owned(),
        })?;

        Ok(V3 { home })
    }
}

impl TryFrom<Empty> for V3 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<Self> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v3::VoltaHome::new(old.home);
        home.create().with_context(|| ErrorKind::CreateDirError {
            dir: home.root().to_owned(),
        })?;

        V3::complete_migration(home)
    }
}

impl TryFrom<V2> for V3 {
    type Error = VoltaError;

    fn try_from(old: V2) -> Fallible<Self> {
        debug!("Migrating from V2 layout");

        let new_home = v3::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|| ErrorKind::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Migrate installed packages to the new workflow
        migrate_packages(&old.home)?;

        // Complete the migration, writing the V3 layout file
        let layout = V3::complete_migration(new_home)?;

        // Remove the V2 layout file, since we're now on V3 (do this after writing the V3 file so that we know the migration succeeded)
        remove_file_if_exists(old.home.layout_file())?;

        Ok(layout)
    }
}

fn migrate_packages(old_home: &v2::VoltaHome) -> Fallible<()> {
    let packages = get_installed_packages(old_home);
    let mut session = Session::init();

    for package in packages {
        migrate_single_package(package, &mut session)?;
    }

    Ok(())
}

/// Determine a list of all installed packages that are using the legacy package config
fn get_installed_packages(old_home: &v2::VoltaHome) -> Vec<LegacyPackageConfig> {
    WalkDir::new(old_home.default_package_dir())
        .max_depth(2)
        .into_iter()
        .filter_map(|res| match res {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    let config = LegacyPackageConfig::from_file(entry.path());

                    if config.is_none() {
                        debug!("Unable to parse config file: {}", entry.path().display());
                    }

                    config
                } else {
                    None
                }
            }
            Err(error) => {
                debug!("Error reading directory entry: {}", error);
                None
            }
        })
        .collect()
}

/// Migrate a single package to the new workflow
///
/// Note: This relies on the package install logic in `volta_core`. If that logic changes, then
/// this migration may need to be updated to accommodate the new end result.
fn migrate_single_package(config: LegacyPackageConfig, session: &mut Session) -> Fallible<()> {
    let tool = Package::new(config.name, VersionSpec::Exact(config.version))?;

    let platform: PlatformSpec = config.platform.into();
    let image = platform.as_binary().checkout(session)?;

    // Run the global install command
    tool.global_install(&image)?;
    // Overwrite the config files and image directory
    tool.complete_install(&image)?;

    Ok(())
}
