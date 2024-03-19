use std::path::PathBuf;

use super::manager::PackageManager;
use super::metadata::{BinConfig, PackageConfig, PackageManifest};
use crate::error::{ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::platform::{Image, PlatformSpec};
use crate::shim;
use crate::tool::check_shim_reachable;

/// Read the manifest for the package being installed
pub(super) fn parse_manifest(
    package_name: &str,
    staging_dir: PathBuf,
    manager: PackageManager,
) -> Fallible<PackageManifest> {
    let mut package_dir = manager.source_dir(staging_dir);
    package_dir.push(package_name);

    PackageManifest::for_dir(package_name, &package_dir)
}

/// Generate configuration files and shims for the package and each of its bins
pub(super) fn write_config_and_shims(
    name: &str,
    manifest: &PackageManifest,
    image: &Image,
    manager: PackageManager,
) -> Fallible<()> {
    validate_bins(name, manifest)?;

    let platform = PlatformSpec {
        node: image.node.value.clone(),
        npm: image.npm.clone().map(|s| s.value),
        pnpm: image.pnpm.clone().map(|s| s.value),
        yarn: image.yarn.clone().map(|s| s.value),
    };

    // Generate the shims and bin configs for each bin provided by the package
    for bin_name in &manifest.bin {
        shim::create(bin_name)?;
        check_shim_reachable(bin_name);

        BinConfig {
            name: bin_name.clone(),
            package: name.into(),
            version: manifest.version.clone(),
            platform: platform.clone(),
            manager,
        }
        .write()?;
    }

    // Write the config for the package
    PackageConfig {
        name: name.into(),
        version: manifest.version.clone(),
        platform,
        bins: manifest.bin.clone(),
        manager,
    }
    .write()?;

    Ok(())
}

/// Validate that we aren't attempting to install a bin that is already installed by
/// another package.
fn validate_bins(package_name: &str, manifest: &PackageManifest) -> Fallible<()> {
    let home = volta_home()?;
    for bin_name in &manifest.bin {
        // Check for name conflicts with already-installed bins
        // Some packages may install bins with the same name
        if let Ok(config) = BinConfig::from_file(home.default_tool_bin_config(bin_name)) {
            // The file exists, so there is a bin with this name
            // That is okay iff it came from the package that is currently being installed
            if package_name != config.package {
                return Err(ErrorKind::BinaryAlreadyInstalled {
                    bin_name: bin_name.into(),
                    existing_package: config.package,
                    new_package: package_name.into(),
                }
                .into());
            }
        }
    }

    Ok(())
}
