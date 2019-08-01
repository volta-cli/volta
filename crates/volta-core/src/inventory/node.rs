use std::collections::BTreeSet;

use regex::Regex;
use semver::Version;

use volta_fail::Fallible;

use super::versions_matching;
use crate::path;

// Convenience for access as `node::Collection`
pub use NodeCollection as Collection;

pub struct NodeCollection {
    pub versions: BTreeSet<Version>,
}

impl NodeCollection {
    pub(crate) fn load() -> Fallible<Self> {
        let re = Regex::new(
            r"(?x)
            node
            -
            v(?P<version>\d+\.\d+\.\d+) # Node version
            -
            (?P<os>[a-z]+)              # operating system
            -
            (?P<arch>[a-z0-9]+)         # architecture
            \.(zip|tar\.gz)
            ",
        )
        .unwrap();

        let versions = versions_matching(&path::node_inventory_dir()?, &re)?;

        Ok(NodeCollection { versions })
    }
}
