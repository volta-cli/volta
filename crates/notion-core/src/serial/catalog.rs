use super::super::catalog;

use std::string::ToString;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::default::Default;

use notion_fail::{Fallible, ResultExt};

use semver::{Version, SemVerError};

#[derive(Serialize, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    node: NodeCatalog
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeCatalog {
    activated: Option<String>,
    versions: Vec<String>
}

impl Default for NodeCatalog {
    fn default() -> Self {
        NodeCatalog {
            activated: None,
            versions: vec![]
        }
    }
}

impl Catalog {
    pub fn into_catalog(self) -> Fallible<catalog::Catalog> {
        Ok(catalog::Catalog {
            node: self.node.into_node_catalog().unknown()?
        })
    }
}

impl NodeCatalog {
    fn into_node_catalog(self) -> Fallible<catalog::NodeCatalog> {
        let activated = match self.activated {
            Some(v) => Some(Version::parse(&v[..]).unknown()?),
            None => None
        };

        let versions: Result<Vec<Version>, SemVerError> = self.versions.into_iter().map(|s| {
            Ok(Version::parse(&s[..])?)
        }).collect();

        Ok(catalog::NodeCatalog {
            activated: activated,
            versions: BTreeSet::from_iter(versions.unknown()?)
        })
    }
}

impl catalog::Catalog {

    pub fn to_serial(&self) -> Catalog {
        Catalog {
            node: self.node.to_serial()
        }
    }

}
impl catalog::NodeCatalog {
    fn to_serial(&self) -> NodeCatalog {
        NodeCatalog {
            activated: self.activated.clone().map(|v| v.to_string()),
            versions: self.versions.iter().map(|v| v.to_string()).collect()
        }
    }
}
