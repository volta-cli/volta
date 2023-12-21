use std::fs::File;
use std::path::Path;

use node_semver::Version;
use volta_core::platform::PlatformSpec;
use volta_core::version::{option_version_serde, version_serde};

#[derive(serde::Deserialize)]
pub struct LegacyPackageConfig {
    pub name: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub platform: LegacyPlatform,
    pub bins: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct LegacyPlatform {
    pub node: NodeVersion,
    #[serde(with = "option_version_serde")]
    pub yarn: Option<Version>,
}

#[derive(serde::Deserialize)]
pub struct NodeVersion {
    #[serde(with = "version_serde")]
    pub runtime: Version,
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
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
            node: config_platform.node.runtime,
            npm: config_platform.node.npm,
            // LegacyPlatform (layout.v2) doesn't have a pnpm field
            pnpm: None,
            yarn: config_platform.yarn,
        }
    }
}
