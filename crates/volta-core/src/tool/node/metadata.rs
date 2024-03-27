use std::collections::HashSet;

use super::NODE_DISTRO_IDENTIFIER;
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "aarch64")
))]
use super::NODE_DISTRO_IDENTIFIER_FALLBACK;
use crate::version::{option_version_serde, version_serde};
use node_semver::Version;
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
                #[cfg(not(any(
                    all(target_os = "macos", target_arch = "aarch64"),
                    all(target_os = "windows", target_arch = "aarch64")
                )))]
                if entry.npm.is_some() && entry.files.contains(NODE_DISTRO_IDENTIFIER) {
                    Some(NodeEntry {
                        version: entry.version,
                        lts: entry.lts,
                    })
                } else {
                    None
                }

                #[cfg(any(
                    all(target_os = "macos", target_arch = "aarch64"),
                    all(target_os = "windows", target_arch = "aarch64")
                ))]
                if entry.npm.is_some()
                    && (entry.files.contains(NODE_DISTRO_IDENTIFIER)
                        || entry.files.contains(NODE_DISTRO_IDENTIFIER_FALLBACK))
                {
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

#[allow(clippy::unnecessary_wraps)] // Needs to match the API expected by Serde
fn lts_version_serde<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
