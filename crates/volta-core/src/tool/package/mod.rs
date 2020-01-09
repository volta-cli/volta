use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

use super::{debug_already_fetched, info_fetched, Tool};
use crate::error::ErrorDetails;
use crate::fs::{delete_dir_error, delete_file_error, dir_entry_match};
use crate::layout::volta_home;
use crate::session::Session;
use crate::shim;
use crate::style::{success_prefix, tool_version};
use dunce::canonicalize;
use log::info;
use semver::Version;
use volta_fail::{Fallible, ResultExt};

mod fetch;
mod install;
mod resolve;
mod serial;

pub use install::{BinConfig, BinLoader, PackageConfig};
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
    canonicalize(raw_path).with_context(|_| ErrorDetails::ExecutablePathError {
        command: bin_name.to_string(),
    })
}

/// Details required for fetching a 3rd-party Package
#[derive(Debug)]
pub struct PackageDetails {
    pub(crate) version: Version,
    pub(crate) tarball_url: String,
    pub(crate) shasum: String,
}

/// The Tool implementation for fetching and installing 3rd-party packages
#[derive(Debug)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) details: PackageDetails,
}

impl Package {
    pub fn new(name: String, details: PackageDetails) -> Self {
        Package { name, details }
    }

    fn fetch_internal(&self, session: &mut Session) -> Fallible<()> {
        // ISSUE(#288) - Once we have a valid Collection, we can check that in the same way as node/yarn
        // Until then, we use the existence of the image directory as the indicator that the package is
        // already fetched
        if volta_home()?
            .package_image_dir(&self.name, &self.details.version.to_string())
            .exists()
        {
            debug_already_fetched(self);
            Ok(())
        } else {
            fetch::fetch(&self.name, &self.details, session)
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
    fn fetch(self, session: &mut Session) -> Fallible<()> {
        self.fetch_internal(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self, session: &mut Session) -> Fallible<()> {
        if self.is_installed() {
            info!("Package {} is already installed", self);
            Ok(())
        } else {
            self.fetch_internal(session)?;

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
    fn pin(self, _session: &mut Session) -> Fallible<()> {
        Err(ErrorDetails::CannotPinPackage { package: self.name }.into())
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
    // if the package config file exists, use that to remove any installed bins and shims
    let package_config_file = home.default_package_config_file(name);
    if package_config_file.exists() {
        let package_config = PackageConfig::from_file(&package_config_file)?;

        for bin_name in package_config.bins {
            remove_config_and_shim(&bin_name, name)?;
        }

        fs::remove_file(&package_config_file)
            .with_context(delete_file_error(&package_config_file))?;
    } else {
        // there is no package config - check for orphaned binaries
        for bin_name in binaries_from_package(name)? {
            remove_config_and_shim(&bin_name, name)?;
        }
    }

    // if any unpacked and initialized packages exists, remove them
    let package_image_dir = home.package_image_root_dir().join(name);
    if package_image_dir.exists() {
        fs::remove_dir_all(&package_image_dir)
            .with_context(delete_dir_error(&package_image_dir))?;
    }

    Ok(())
}

fn remove_config_and_shim(bin_name: &str, pkg_name: &str) -> Fallible<()> {
    shim::delete(bin_name)?;
    let config_file = volta_home()?.default_tool_bin_config(&bin_name);
    fs::remove_file(&config_file).with_context(delete_file_error(&config_file))?;
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
    if bin_config_dir.exists() {
        dir_entry_match(&bin_config_dir, |entry| {
            let path = entry.path();
            if let Ok(config) = BinConfig::from_file(path) {
                if config.package == package {
                    return Some(config.name);
                }
            };
            None
        })
        .with_context(|_| ErrorDetails::ReadBinConfigDirError {
            dir: bin_config_dir.to_owned(),
        })
    } else {
        Ok(vec![])
    }
}
