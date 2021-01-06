use std::collections::HashSet;
use std::str::FromStr;

use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Deserializer};
use serde_json::Error;

/// The index of the public Node server.
pub struct NodeIndex {
    pub(super) entries: Vec<NodeEntry>,
}

impl FromStr for NodeIndex {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw: RawNodeIndex = serde_json::de::from_str(s)?;
        Ok(raw.into())
    }
}

#[derive(Debug)]
pub struct NodeEntry {
    pub version: Version,
    pub npm: Version,
    pub files: NodeDistroFiles,
    pub lts: bool,
}

/// The set of available files on the public Node server for a given Node version.
#[derive(Debug)]
pub struct NodeDistroFiles {
    pub files: HashSet<String>,
}

#[derive(Deserialize)]
pub struct RawNodeIndex(Vec<RawNodeEntry>);

#[derive(Deserialize)]
pub struct RawNodeEntry {
    #[serde(with = "version_serde")]
    version: Version,
    #[serde(default)] // handles Option
    #[serde(with = "option_version_serde")]
    npm: Option<Version>,
    files: Vec<String>,
    #[serde(deserialize_with = "lts_version_serde")]
    lts: bool,
}

impl From<RawNodeIndex> for NodeIndex {
    fn from(raw: RawNodeIndex) -> NodeIndex {
        let mut entries = Vec::new();
        for entry in raw.0 {
            if let Some(npm) = entry.npm {
                let data = NodeDistroFiles {
                    files: entry.files.into_iter().collect(),
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
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
