use super::metadata::{BinConfig, PackageConfig};
use crate::error::AcceptableErrorToDefault;
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{dir_entry_match, remove_dir_if_exists, remove_file_if_exists};
use crate::layout::volta_home;
use crate::shim;
use crate::style::success_prefix;
use crate::sync::VoltaLock;
use log::{info, warn};

/// Uninstalls the specified package.
///
/// This removes:
///
/// - The JSON configuration files for both the package and its bins
/// - The shims for the package bins
/// - The package directory itself
pub fn uninstall(name: &str) -> Fallible<()> {
    let home = volta_home()?;
    // Acquire a lock on the Volta directory, if possible, to prevent concurrent changes
    let _lock = VoltaLock::acquire();

    // If the package config file exists, use that to remove any installed bins and shims
    let package_config_file = home.default_package_config_file(name);

    let package_found = match PackageConfig::from_file(&package_config_file) {
        Err(error) => {
            if error.is_not_found_error_kind() {
                let package_binary_list = binaries_from_package(name)?;
                if !package_binary_list.is_empty() {
                    for bin_name in package_binary_list {
                        remove_config_and_shim(&bin_name, name)?;
                    }
                    true
                } else {
                    false
                }
            } else {
                return Err(error);
            }
        }
        Ok(package_config) => {
            for bin_name in package_config.bins {
                remove_config_and_shim(&bin_name, name)?;
            }

            remove_file_if_exists(package_config_file)?;
            true
        }
    };

    // Remove the package directory itself
    let package_image_dir = home.package_image_dir(name);
    remove_dir_if_exists(package_image_dir)?;

    if package_found {
        info!("{} package '{}' uninstalled", success_prefix(), name);
    } else {
        warn!("No package '{}' found to uninstall", name);
    }

    Ok(())
}

/// Remove a shim and its associated configuration file
fn remove_config_and_shim(bin_name: &str, pkg_name: &str) -> Fallible<()> {
    shim::delete(bin_name)?;
    let config_file = volta_home()?.default_tool_bin_config(bin_name);
    remove_file_if_exists(config_file)?;
    info!(
        "Removed executable '{}' installed by '{}'",
        bin_name, pkg_name
    );
    Ok(())
}

/// Reads the contents of a directory and returns a Vec containing the names of
/// all the binaries installed by the given package.
fn binaries_from_package(package: &str) -> Fallible<Vec<String>> {
    let bin_config_dir = volta_home()?.default_bin_dir();

    dir_entry_match(&bin_config_dir, |entry| {
        let path = entry.path();
        if let Ok(config) = BinConfig::from_file(path) {
            if config.package == package {
                return Some(config.name);
            }
        }
        None
    })
    .with_context(|| ErrorKind::ReadBinConfigDirError {
        dir: bin_config_dir.to_owned(),
    })
    .accept_error_as_default_if(|e| e.is_not_found_error_kind())
}
