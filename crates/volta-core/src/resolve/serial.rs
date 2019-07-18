use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter::FromIterator;

use crate::tool::PackageDetails;
use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Deserializer};
use volta_fail::Fallible;

#[derive(Deserialize)]
pub struct RawNodeIndex(Vec<RawNodeEntry>);

#[derive(Deserialize)]
pub struct RawNodeEntry {
    #[serde(with = "version_serde")]
    pub version: Version,
    #[serde(default)] // handles Option
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
    pub files: Vec<String>,
    #[serde(deserialize_with = "lts_version_serde")]
    pub lts: bool,
}

impl RawNodeIndex {
    pub fn into_index(self) -> Fallible<super::node::NodeIndex> {
        let mut entries = Vec::new();
        for entry in self.0 {
            if let Some(npm) = entry.npm {
                let data = super::node::NodeDistroFiles {
                    files: HashSet::from_iter(entry.files.into_iter()),
                };
                entries.push(super::node::NodeEntry {
                    version: entry.version,
                    npm,
                    files: data,
                    lts: entry.lts,
                });
            }
        }
        Ok(super::node::NodeIndex { entries })
    }
}

fn lts_version_serde<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(_t) => Ok(true),
        Err(_e) => Ok(false),
    }
}

#[derive(Deserialize)]
pub struct RawYarnIndex(Vec<RawYarnEntry>);

#[derive(Deserialize)]
pub struct RawYarnEntry {
    /// Yarn releases are given a tag name of the form "v$version" where $version
    /// is the release's version string.
    #[serde(with = "version_serde")]
    pub tag_name: Version,

    /// The GitHub API provides a list of assets. Some Yarn releases don't include
    /// a tarball, so we don't support them and remove them from the set of available
    /// Yarn versions.
    pub assets: Vec<RawYarnAsset>,
}

impl RawYarnEntry {
    /// Is this entry a full release, i.e., does this entry's asset list include a
    /// proper release tarball?
    fn is_full_release(&self) -> bool {
        let release_filename = &format!("yarn-v{}.tar.gz", self.tag_name)[..];
        self.assets
            .iter()
            .any(|&RawYarnAsset { ref name }| name == release_filename)
    }
}

#[derive(Deserialize)]
pub struct RawYarnAsset {
    /// The filename of an asset included in a Yarn GitHub release.
    pub name: String,
}

impl RawYarnIndex {
    pub fn into_index(self) -> Fallible<super::yarn::YarnIndex> {
        let mut entries = BTreeSet::new();
        for entry in self.0 {
            if entry.is_full_release() {
                entries.insert(entry.tag_name);
            }
        }
        Ok(super::yarn::YarnIndex { entries })
    }
}

/// Package Metadata Response
///
/// See npm registry API doc:
/// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md
#[derive(Deserialize, Debug)]
pub struct RawPackageMetadata {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, RawPackageVersionInfo>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: RawPackageDistTags,
}

#[derive(Deserialize, Debug)]
pub struct RawPackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawPackageDistTags {
    #[serde(with = "version_serde")]
    pub latest: Version,
    pub beta: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawDistInfo {
    pub shasum: String,
    pub tarball: String,
}

impl From<RawPackageMetadata> for super::package::PackageIndex {
    fn from(serial: RawPackageMetadata) -> super::package::PackageIndex {
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

        super::package::PackageIndex {
            latest: serial.dist_tags.latest,
            entries,
        }
    }
}

// Data structures for `npm view` data
//
// $ npm view --json gulp@latest
// {
//   "name": "gulp",
//   "description": "The streaming build system.",
//   "dist-tags": {
//     "latest": "4.0.2"
//   },
//   "version": "4.0.2",
//   "engines": {
//     "node": ">= 0.10"
//   },
//   "dist": {
//     "shasum": "543651070fd0f6ab0a0650c6a3e6ff5a7cb09caa",
//     "tarball": "https://registry.npmjs.org/gulp/-/gulp-4.0.2.tgz",
//   },
//   (...and lots of other stuff we don't use...)
// }
//
#[derive(Deserialize, Clone, Debug)]
pub struct NpmViewData {
    pub name: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
    #[serde(rename = "dist-tags")]
    pub dist_tags: RawPackageDistTags,
}

impl From<NpmViewData> for PackageDetails {
    fn from(view_data: NpmViewData) -> PackageDetails {
        PackageDetails {
            version: view_data.version,
            tarball_url: view_data.dist.tarball,
            shasum: view_data.dist.shasum,
        }
    }
}
