use std::collections::HashSet;
use std::iter::FromIterator;

use super::resolve::{NodeIndex, NodeDistroFiles, NodeEntry};
use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Deserializer};

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

impl From<RawNodeIndex> for NodeIndex {
    fn from(raw: RawNodeIndex) -> NodeIndex {
        let mut entries = Vec::new();
        for entry in raw.0 {
            if let Some(npm) = entry.npm {
                let data = NodeDistroFiles {
                    files: HashSet::from_iter(entry.files.into_iter()),
                };
                entries.push(NodeEntry {
                    version: entry.version,
                    npm,
                    files: data,
                    lts: entry.lts,
                });
            }
        }
        NodeIndex { entries }
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
