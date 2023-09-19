use std::fs::File;
use std::path::Path;

use semver::Version;
use volta_core::platform::PlatformSpec;
use volta_core::tool::package::PackageManager;
use volta_core::version::{option_version_serde, version_serde};

#[derive(serde::Deserialize)]
pub struct LegacyPackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// The platform used to install this package
    pub platform: LegacyPlatform,
    /// The binaries installed by this package
    pub bins: Vec<String>,
    /// The package manager that was used to install this package
    pub manager: PackageManager,
}

#[derive(serde::Deserialize)]
pub struct LegacyPlatform {
    #[serde(with = "version_serde")]
    pub node: Version,
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
    #[serde(with = "option_version_serde")]
    pub pnpm: Option<Version>,
    #[serde(with = "option_version_serde")]
    pub yarn: Option<Version>,
}

impl LegacyPackageConfig {
    pub fn from_file(config_file: &Path) -> Option<Self> {
        let file = File::open(config_file).ok()?;

        serde_json::from_reader(file).ok()
    }
}

impl From<LegacyPlatform> for PlatformSpec {
    fn from(config_platform: LegacyPlatform) -> Self {
        PlatformSpec {
            node: config_platform.node,
            npm: config_platform.npm,
            pnpm: config_platform.pnpm,
            yarn: config_platform.yarn,
        }
    }
}
