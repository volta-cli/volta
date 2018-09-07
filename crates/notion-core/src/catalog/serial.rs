use std::collections::{BTreeSet, HashSet};
use std::default::Default;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::string::ToString;

use notion_fail::{Fallible, ResultExt};

use semver::{SemVerError, Version};

#[derive(Serialize, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    node: NodeCollection,
    #[serde(default)]
    yarn: YarnCollection,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeCollection {
    default: Option<String>,
    versions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "yarn")]
pub struct YarnCollection {
    default: Option<String>,
    versions: Vec<String>,
}

impl Default for NodeCollection {
    fn default() -> Self {
        NodeCollection {
            default: None,
            versions: vec![],
        }
    }
}

impl Default for YarnCollection {
    fn default() -> Self {
        YarnCollection {
            default: None,
            versions: vec![],
        }
    }
}

impl Catalog {
    pub fn into_catalog(self) -> Fallible<super::Catalog> {
        Ok(super::Catalog {
            node: self.node.into_node_collection().unknown()?,
            yarn: self.yarn.into_yarn_collection().unknown()?,
        })
    }
}

impl NodeCollection {
    fn into_node_collection(self) -> Fallible<super::NodeCollection> {
        let default = match self.default {
            Some(v) => Some(Version::parse(&v[..]).unknown()?),
            None => None,
        };

        let versions: Result<Vec<Version>, SemVerError> = self.versions
            .into_iter()
            .map(|s| Ok(Version::parse(&s[..])?))
            .collect();

        Ok(super::NodeCollection {
            default,
            versions: BTreeSet::from_iter(versions.unknown()?),
            phantom: PhantomData,
        })
    }
}

impl YarnCollection {
    fn into_yarn_collection(self) -> Fallible<super::YarnCollection> {
        let default = match self.default {
            Some(v) => Some(Version::parse(&v[..]).unknown()?),
            None => None,
        };

        let versions: Result<Vec<Version>, SemVerError> = self.versions
            .into_iter()
            .map(|s| Ok(Version::parse(&s[..])?))
            .collect();

        Ok(super::YarnCollection {
            default,
            versions: BTreeSet::from_iter(versions.unknown()?),
            phantom: PhantomData,
        })
    }
}

impl super::Catalog {
    pub fn to_serial(&self) -> Catalog {
        Catalog {
            node: self.node.to_serial(),
            yarn: self.yarn.to_serial(),
        }
    }
}
impl super::NodeCollection {
    fn to_serial(&self) -> NodeCollection {
        NodeCollection {
            default: self.default.clone().map(|v| v.to_string()),
            versions: self.versions.iter().map(|v| v.to_string()).collect(),
        }
    }
}

impl super::YarnCollection {
    fn to_serial(&self) -> YarnCollection {
        YarnCollection {
            default: self.default.clone().map(|v| v.to_string()),
            versions: self.versions.iter().map(|v| v.to_string()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Index(Vec<Entry>);

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub version: String,
    pub files: Vec<String>,
}

impl Index {
    pub fn into_index(self) -> Fallible<super::Index> {
        let mut entries = Vec::new();
        for entry in self.0 {
            let data = super::VersionData {
                files: HashSet::from_iter(entry.files.into_iter()),
            };
            let mut version = &entry.version[..];
            version = version.trim();
            if version.starts_with('v') {
                version = &version[1..];
            }
            entries.push((Version::parse(version).unknown()?, data));
        }
        Ok(super::Index { entries })
    }
}
