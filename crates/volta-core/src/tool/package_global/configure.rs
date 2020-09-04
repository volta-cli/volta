use super::metadata::PackageManifest;
use super::Package;
use crate::error::Fallible;
use crate::platform::Image;

impl Package {
    pub(super) fn write_config_and_shims(
        &self,
        _manifest: &PackageManifest,
        _image: &Image,
    ) -> Fallible<()> {
        // TODO: Generate Package config and write it
        // TODO: Generate bin configs and write them
        // TODO: Create shims for bins
        Ok(())
    }

    pub(super) fn parse_manifest(&self) -> Fallible<PackageManifest> {
        let mut package_dir = self.staging.path().join(&self.name);
        // Looking forward to allowing direct package manager global installs,
        // this method should be updated to support the different directory layouts
        // of different package managers.

        // `lib` is not in the directory structure on Windows
        #[cfg(not(windows))]
        package_dir.push("lib");

        package_dir.push("node_modules");
        package_dir.push(&self.name);

        let mut manifest = PackageManifest::for_dir(&self.name, &package_dir)?;

        // Internally, an empty string key represents a bin entry that is only the PATH
        // This needs to be updated to use the package name as the key
        if let Some(path) = manifest.bin.remove("") {
            manifest.bin.insert(self.name.clone(), path);
        }

        Ok(manifest)
    }
}
