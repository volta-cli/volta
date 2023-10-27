use std::convert::TryFrom;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::empty::Empty;
use crate::v3::V3;
use log::{debug, warn};
use volta_core::error::{Context, ErrorKind, Fallible, VoltaError};
use volta_core::fs::remove_file_if_exists;
use volta_core::platform::PlatformSpec;
use volta_core::session::Session;
use volta_core::tool::{Package, PackageConfig};
use volta_core::version::VersionSpec;
use volta_layout::{v3, v4};
use walkdir::WalkDir;

mod config;

use config::LegacyPackageConfig;

/// Represents a V4 Volta layout (used by Volta v1.1.2 and above)
///
/// Holds a reference to the V4 layout struct to support future migrations
pub struct V4 {
    pub home: v4::VoltaHome,
}

impl V4 {
    pub fn new(home: PathBuf) -> Self {
        V4 {
            home: v4::VoltaHome::new(home),
        }
    }

    /// Write the layout file to mark migration to V3 as complete
    ///
    /// Should only be called once all other migration steps are finished, so that we don't
    /// accidentally mark an incomplete migration as completed
    fn complete_migration(home: v4::VoltaHome) -> Fallible<Self> {
        debug!("Writing layout marker file");
        File::create(home.layout_file()).with_context(|| ErrorKind::CreateLayoutFileError {
            file: home.layout_file().to_owned(),
        })?;

        Ok(V4 { home })
    }
}

impl TryFrom<Empty> for V4 {
    type Error = VoltaError;

    fn try_from(old: Empty) -> Fallible<Self> {
        debug!("New Volta installation detected, creating fresh layout");

        let home = v4::VoltaHome::new(old.home);
        home.create().with_context(|| ErrorKind::CreateDirError {
            dir: home.root().to_owned(),
        })?;

        V4::complete_migration(home)
    }
}

impl TryFrom<V3> for V4 {
    type Error = VoltaError;

    fn try_from(old: V3) -> Fallible<Self> {
        debug!("Migrating from V3 layout");

        let new_home = v4::VoltaHome::new(old.home.root().to_owned());
        new_home
            .create()
            .with_context(|| ErrorKind::CreateDirError {
                dir: new_home.root().to_owned(),
            })?;

        // Migrate installed packages to the new workflow
        migrate_packages(&old.home)?;

        // Complete the migration, writing the V4 layout file
        let layout = V4::complete_migration(new_home)?;

        // Remove the V3 layout file, since we're now on V4 (do this after writing the V4 file so that we know the migration succeeded)
        remove_file_if_exists(old.home.layout_file())?;

        Ok(layout)
    }
}

fn migrate_packages(old_home: &v3::VoltaHome) -> Fallible<()> {
    let packages = get_installed_packages(old_home);
    let mut session = Session::init();

    for package in packages {
        migrate_single_package(package, &mut session)?;
    }

    Ok(())
}

/// Determine a list of all installed packages that are using the legacy package config
fn get_installed_packages(old_home: &v3::VoltaHome) -> Vec<LegacyPackageConfig> {
    WalkDir::new(old_home.default_package_dir())
        .max_depth(2)
        .into_iter()
        .filter_map(|res| match res {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    let config = LegacyPackageConfig::from_file(entry.path());

                    // If unable to parse the config file and this isn't an already-migrated
                    // package, then show debug information and a warning for the user.
                    if config.is_none() && !is_migrated_config(entry.path()) {
                        debug!("Unable to parse config file: {}", entry.path().display());
                        if let Some(name) = entry.path().file_stem() {
                            let name = name.to_string_lossy();
                            warn!(
                                "Could not migrate {}. Please run `volta install {0}` to migrate the package manually.",
                                name
                            );
                        }
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

/// Determine if a package has already been migrated by attempting to read the V4 PackageConfig
fn is_migrated_config(config_path: &Path) -> bool {
    PackageConfig::from_file(config_path).is_ok()
}

/// Migrate a single package to the new workflow
///
/// Note: This relies on the package install logic in `volta_core`. If that logic changes, then
/// the end result may not be a valid V4 layout any more, and this migration will need to be
/// updated. Specifically, the invariants we rely on are:
///
/// - Package image directory is in the same location
/// - Package config files are in the same location and the same format
/// - Binary config files are in the same location and the same format
///
/// If any of those are violated, this migration may be invalid and need to be reworked / scrapped
fn migrate_single_package(config: LegacyPackageConfig, session: &mut Session) -> Fallible<()> {
    let tool = Package::new(config.name, VersionSpec::Exact(config.version))?;

    let platform: PlatformSpec = config.platform.into();
    let image = platform.as_binary().checkout(session)?;

    // Run the global install command
    tool.run_install(&image)?;
    // Overwrite the config files and image directory
    tool.complete_install(&image)?;

    Ok(())
}
