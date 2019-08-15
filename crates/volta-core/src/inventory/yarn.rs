use std::collections::BTreeSet;

use regex::Regex;
use semver::Version;

use volta_fail::Fallible;

use super::versions_matching;
use crate::path;

// Convenience for access as `yarn::Collection`
pub use YarnCollection as Collection;

pub struct YarnCollection {
    pub versions: BTreeSet<Version>,
}

impl YarnCollection {
    pub(crate) fn load() -> Fallible<Self> {
        let re = Regex::new(
            r"(?x)
            yarn
            -
            v(?P<version>\d+\.\d+\.\d+) # Yarn version
            \.tar\.gz
            ",
        )
        .unwrap();

        let versions = versions_matching(&path::yarn_inventory_dir()?, &re)?;

        Ok(Collection { versions })
    }
}
