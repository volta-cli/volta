use std::collections::BTreeSet;

use semver::Version;
use volta_fail::Fallible;

use super::{versions_matching, Collection};
use crate::{path, tool::Yarn};
use regex::Regex;

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

impl super::Collection for YarnCollection {
    type Tool = Yarn;

    fn add(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }

    fn remove(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }
}

// So users can do `yarn::Collection`.
pub use YarnCollection as Collection;
