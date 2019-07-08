use std::collections::{BTreeSet, HashSet};
use std::iter::FromIterator;

use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};
use volta_fail::Fallible;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawNodeIndex(Vec<RawNodeEntry>);

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct RawYarnIndex(Vec<RawYarnEntry>);

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
