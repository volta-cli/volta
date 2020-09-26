use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use super::registry::PackageDetails;
use super::{check_fetched, debug_already_fetched, info_fetched, FetchStatus, Tool};
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{dir_entry_match, ok_if_not_found, remove_dir_if_exists, remove_file_if_exists};
use crate::inventory::package_available;
use crate::layout::volta_home;
use crate::session::Session;
use crate::shim;
use crate::style::{success_prefix, tool_version};
use crate::sync::VoltaLock;
use dunce::canonicalize;
use log::{info, warn};
use semver::Version;

mod fetch;
mod install;
pub(crate) mod metadata;
pub(crate) mod resolve;

pub use metadata::{BinConfig, BinLoader, PackageConfig};
pub use resolve::resolve;

pub fn bin_full_path<P>(
    package: &str,
    version: &Version,
    bin_name: &str,
    bin_path: P,
) -> Fallible<PathBuf>
where
    P: AsRef<Path>,
{
    let raw_path = volta_home()?
        .package_image_dir(package, &version.to_string())
        .join(bin_path);

    // canonicalize because path is relative, and sometimes uses '.' char
    canonicalize(raw_path).with_context(|| ErrorKind::ExecutablePathError {
        command: bin_name.to_string(),
    })
}

/// The Tool implementation for fetching and installing 3rd-party packages
pub struct Package {
    pub(crate) name: String,
    pub(crate) details: PackageDetails,
}

impl Package {
    pub fn new(name: String, details: PackageDetails) -> Self {
        Package { name, details }
    }

    fn ensure_fetched(&self, session: &mut Session) -> Fallible<()> {
        match check_fetched(|| package_available(&self.name, &self.details.version))? {
            FetchStatus::AlreadyFetched => {
                debug_already_fetched(self);
                Ok(())
            }
            FetchStatus::FetchNeeded(_lock) => fetch::fetch(&self.name, &self.details, session),
        }
    }

    fn is_installed(&self) -> bool {
        // Check if the package config exists and contains the same version
        // (The PackageConfig is written after the installation is complete)
        if let Ok(home) = volta_home() {
            let pkg_config_file = home.default_package_config_file(&self.name);
            if let Ok(package_config) = PackageConfig::from_file(&pkg_config_file) {
                return package_config.version == self.details.version;
            }
        }
        false
    }
}

impl Tool for Package {
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        // Acquire a lock on the Volta directory, if possible, to prevent concurrent changes
        let _lock = VoltaLock::acquire();
        if self.is_installed() {
            info!("Package {} is already installed", self);
            Ok(())
        } else {
            self.ensure_fetched(session)?;

            let bin_map = install::install(&self.name, &self.details.version, session)?;

            let bins = bin_map
                .keys()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(", ");
            info!(
                "{} installed {} with executables: {}",
                success_prefix(),
                self,
                bins
            );
            Ok(())
        }
    }
    fn pin(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        Err(ErrorKind::CannotPinPackage { package: self.name }.into())
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version(&self.name, &self.details.version))
    }
}

/// Uninstall the specified package.
///
/// This removes:
/// * the json config files
/// * the shims
/// * the unpacked and initialized package
pub fn uninstall(name: &str) -> Fallible<()> {
    let home = volta_home()?;
    // Acquire a lock on the Volta directory, if possible, to prevent concurrent changes
    let _lock = VoltaLock::acquire();

    // if the package config file exists, use that to remove any installed bins and shims
    let package_config_file = home.default_package_config_file(name);

    let package_found = match PackageConfig::from_file_if_exists(&package_config_file)? {
        None => {
            // there is no package config - check for orphaned binaries
            let package_binary_list = binaries_from_package(name)?;
            if !package_binary_list.is_empty() {
                for bin_name in package_binary_list {
                    remove_config_and_shim(&bin_name, name)?;
                }
                true
            } else {
                false
            }
        }
        Some(package_config) => {
            for bin_name in package_config.bins {
                remove_config_and_shim(&bin_name, name)?;
            }
            remove_file_if_exists(package_config_file)?;
            true
        }
    };

    // if any unpacked and initialized packages exists, remove them
    let package_image_dir = home.package_image_root_dir().join(name);
    remove_dir_if_exists(package_image_dir)?;

    if package_found {
        info!("{} package '{}' uninstalled", success_prefix(), name);
    } else {
        warn!("No package '{}' found to uninstall", name);
    }

    Ok(())
}

fn remove_config_and_shim(bin_name: &str, pkg_name: &str) -> Fallible<()> {
    shim::delete(bin_name)?;
    let config_file = volta_home()?.default_tool_bin_config(&bin_name);
    remove_file_if_exists(config_file)?;
    info!(
        "Removed executable '{}' installed by '{}'",
        bin_name, pkg_name
    );
    Ok(())
}

/// Reads the contents of a directory and returns a Vec containing the names of
/// all the binaries installed by the input package.
fn binaries_from_package(package: &str) -> Fallible<Vec<String>> {
    let bin_config_dir = volta_home()?.default_bin_dir();
    dir_entry_match(&bin_config_dir, |entry| {
        let path = entry.path();
        if let Ok(config) = BinConfig::from_file(path) {
            if config.package == package {
                return Some(config.name);
            }
        };
        None
    })
    .or_else(ok_if_not_found)
    .with_context(|| ErrorKind::ReadBinConfigDirError {
        dir: bin_config_dir.to_owned(),
    })
}
