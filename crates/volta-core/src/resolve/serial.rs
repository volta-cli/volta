use std::collections::HashSet;
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
