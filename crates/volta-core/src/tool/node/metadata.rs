use std::collections::HashSet;

use super::NODE_DISTRO_IDENTIFIER;
use crate::version::{option_version_serde, version_serde};
use semver::Version;
use serde::{Deserialize, Deserializer};

/// The index of the public Node server.
pub struct NodeIndex {
    pub(super) entries: Vec<NodeEntry>,
}

#[derive(Debug)]
pub struct NodeEntry {
    pub version: Version,
    pub lts: bool,
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
    files: HashSet<String>,
    #[serde(deserialize_with = "lts_version_serde")]
    lts: bool,
}

impl From<RawNodeIndex> for NodeIndex {
    fn from(raw: RawNodeIndex) -> NodeIndex {
        let entries = raw
            .0
            .into_iter()
            .filter_map(|entry| {
                if entry.npm.is_some() && entry.files.contains(NODE_DISTRO_IDENTIFIER) {
                    Some(NodeEntry {
                        version: entry.version,
                        lts: entry.lts,
                    })
                } else {
                    None
                }
            })
            .collect();

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
