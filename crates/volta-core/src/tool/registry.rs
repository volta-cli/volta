use std::collections::HashMap;

use crate::version::{hashmap_version_serde, version_serde};
use semver::Version;
use serde::Deserialize;

/// Details about a package in the npm Registry
#[derive(Debug)]
pub struct PackageDetails {
    pub(crate) version: Version,
    pub(crate) tarball_url: String,
    pub(crate) shasum: String,
}

/// Index of versions of a specific package from the npm Registry
pub struct PackageIndex {
    pub tags: HashMap<String, Version>,
    pub entries: Vec<PackageDetails>,
}

/// Package Metadata Response
///
/// See npm registry API doc:
/// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md
#[derive(Deserialize, Debug)]
pub struct RawPackageMetadata {
    pub name: String,
    pub versions: HashMap<String, RawPackageVersionInfo>,
    #[serde(
        rename = "dist-tags",
        deserialize_with = "hashmap_version_serde::deserialize"
    )]
    pub dist_tags: HashMap<String, Version>,
}

#[derive(Deserialize, Debug)]
pub struct RawPackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawDistInfo {
    pub shasum: String,
    pub tarball: String,
}

impl From<RawPackageMetadata> for PackageIndex {
    fn from(serial: RawPackageMetadata) -> PackageIndex {
        let mut entries: Vec<PackageDetails> = serial
            .versions
            .into_iter()
            .map(|(_, version_info)| PackageDetails {
                version: version_info.version,
                tarball_url: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            })
            .collect();

        entries.sort_by(|a, b| b.version.cmp(&a.version));

        PackageIndex {
            tags: serial.dist_tags,
            entries,
        }
    }
}
