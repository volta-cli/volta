// Serialization for npm package information

use std::collections::HashMap;

use notion_fail::Fallible;
use notion_fail::ResultExt;
use semver::Version;
use serde::{Deserialize, Serialize};


// see npm registry API doc:
// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetadata {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, PackageVersionInfo>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: PackageDistTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    pub version: String,
    pub dist: DistInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDistTags {
    pub latest: String,
    pub beta: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DistInfo {
    pub shasum: String,
    pub tarball: String,
}

impl PackageMetadata {
    pub fn into_index(self) -> Fallible<super::PackageIndex> {
        let latest_version = Version::parse(&self.dist_tags.latest).unknown()?;

        let mut entries = Vec::new();
        for (_, version_info) in self.versions {
            let parsed_version = Version::parse(&version_info.version).unknown()?;
            let entry = super::PackageEntry {
                version: parsed_version,
                tarball: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            };
            entries.push(entry);
        }

        // sort entries by version, largest to smallest
        entries.sort_by(|a, b| a.version.cmp(&b.version).reverse());

        Ok(super::PackageIndex{ latest: latest_version, entries: entries })
    }
}

