use super::metadata::{BinConfig, PackageConfig, PackageManifest};
use super::Package;
use crate::error::{ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::platform::{Image, PlatformSpec};
use crate::shim;

impl Package {
    /// Read the manifest for the package being installed
    pub(super) fn parse_manifest(&self) -> Fallible<PackageManifest> {
        let mut package_dir = self.staging.path().to_owned();
        // Looking forward to allowing direct package manager global installs,
        // this method should be updated to support the different directory layouts
        // of different package managers.

        // `lib` is not in the directory structure on Windows
        #[cfg(not(windows))]
        package_dir.push("lib");

        package_dir.push("node_modules");
        package_dir.push(&self.name);

        PackageManifest::for_dir(&self.name, &package_dir)
    }

    /// Generate configuration files and shims for the package and each of its bins
    pub(super) fn write_config_and_shims(
        &self,
        manifest: &PackageManifest,
        image: &Image,
    ) -> Fallible<()> {
        self.validate_bins(manifest)?;

        let platform = PlatformSpec {
            node: image.node.value.clone(),
            npm: image.npm.clone().map(|s| s.value),
            yarn: image.yarn.clone().map(|s| s.value),
        };

        // Generate the shims and bin configs for each bin provided by the package
        for bin_name in &manifest.bin {
            shim::create(&bin_name)?;

            BinConfig {
                name: bin_name.clone(),
                package: self.name.clone(),
                version: manifest.version.clone(),
                platform: platform.clone(),
            }
            .write()?;
        }

        // Write the config for the package
        PackageConfig {
            name: self.name.clone(),
            version: manifest.version.clone(),
            platform,
            bins: manifest.bin.clone(),
        }
        .write()?;

        Ok(())
    }

    /// Validate that we aren't attempting to install a bin that is already installed by
    /// another package.
    fn validate_bins(&self, manifest: &PackageManifest) -> Fallible<()> {
        let home = volta_home()?;
        for bin_name in &manifest.bin {
            // Check for name conflicts with already-installed bins
            // Some packages may install bins with the same name
            if let Ok(config) = BinConfig::from_file(home.default_tool_bin_config(&bin_name)) {
                // The file exists, so there is a bin with this name
                // That is okay iff it came from the package that is currently being installed
                if self.name != config.package {
                    return Err(ErrorKind::BinaryAlreadyInstalled {
                        bin_name: bin_name.into(),
                        existing_package: config.package,
                        new_package: self.name.clone(),
                    }
                    .into());
                }
            }
        }

        Ok(())
    }
}
