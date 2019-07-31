use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

use super::{info_fetched, info_installed, Tool};
use crate::error::ErrorDetails;
use crate::fs::{delete_dir_error, delete_file_error, dir_entry_match};
use crate::path;
use crate::platform::PlatformSpec;
use crate::session::Session;
use crate::shim;
use crate::style::tool_version;
use log::info;
use semver::Version;
use volta_fail::{Fallible, ResultExt};

mod fetch;
mod serial;

pub use fetch::{BinConfig, BinLoader};

/// Configuration information about an installed package.
///
/// This information will be stored in ~/.volta/tools/user/packages/<package>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "version": "1.4.0",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null
///   },
///   "bins": [
///     "cowsay",
///     "cowthink"
///   ]
/// }
pub struct PackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    pub version: Version,
    /// The platform used to install this package
    pub platform: PlatformSpec,
    /// The binaries installed by this package
    pub bins: Vec<String>,
}

pub fn bin_full_path<P>(
    package: &str,
    version: &Version,
    bin_name: &str,
    bin_path: P,
) -> Fallible<PathBuf>
where
    P: AsRef<Path>,
{
    // canonicalize because path is relative, and sometimes uses '.' char
    path::package_image_dir(package, &version.to_string())?
        .join(bin_path)
        .canonicalize()
        .with_context(|_| ErrorDetails::ExecutablePathError {
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
    pub(super) name: String,
    pub(super) details: PackageDetails,
}

impl Package {
    pub fn new(name: String, details: PackageDetails) -> Self {
        Package { name, details }
    }

    fn fetch_internal(&self) -> Fallible<()> {
        // Check if it's already available? Not sure how exactly
        //debug_already_fetched(self);
        fetch::fetch(&self.name, &self.details)?;

        Ok(())
    }
}

impl Tool for Package {
    fn fetch(self, _session: &mut Session) -> Fallible<()> {
        self.fetch_internal()?;

        // TODO - CPIERCE: Determine how we want to display the information to the user
        info_fetched(self);
        Ok(())
    }
    fn install(self, _session: &mut Session) -> Fallible<()> {
        self.fetch_internal()?;

        // TODO - CPIERCE: Determine how to show the info to the user
        //                 Also do whatever extra steps are needed (npm install, etc)
        info_installed(self);
        Ok(())
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
    // if the package config file exists, use that to remove any installed bins and shims
    let package_config_file = path::user_package_config_file(name)?;
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
    let package_image_dir = path::package_image_root_dir()?.join(name);
    if package_image_dir.exists() {
        fs::remove_dir_all(&package_image_dir)
            .with_context(delete_dir_error(&package_image_dir))?;
    }

    Ok(())
}

fn remove_config_and_shim(bin_name: &str, pkg_name: &str) -> Fallible<()> {
    shim::delete(bin_name)?;
    let config_file = path::user_tool_bin_config(&bin_name)?;
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
    let bin_config_dir = path::user_bin_dir()?;
    if bin_config_dir.exists() {
        dir_entry_match(&bin_config_dir, |entry| {
            let path = entry.path();
            if let Ok(config) = BinConfig::from_file(path) {
                if config.package == package.to_string() {
                    return Some(config.name);
                }
            };
            None
        })
        .with_context(|_| ErrorDetails::ReadBinConfigDirError {
            dir: bin_config_dir,
        })
    } else {
        Ok(vec![])
    }
}
