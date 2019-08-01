use std::collections::BTreeSet;

use semver::Version;

use volta_fail::Fallible;

use crate::tool::{Package, PackageConfig};

// Convenience for access as `package::Collection`
pub use PackageCollection as Collection;

pub struct PackageCollection {
    pub packages: BTreeSet<PackageConfig>,
}

impl PackageCollection {
    // loads an empty PackageCollection
    // ISSUE(#288) Collection only supports versions - for packages we also need names
    pub(crate) fn load() -> Fallible<Self> {
        Ok(PackageCollection {
            packages: BTreeSet::new(),
        })
    }

    pub(crate) fn contains(&self, name: &str, version: &Version) -> bool {
        self.packages
            .iter()
            .find(|config| config.name == name && &config.version == version)
            .is_some()
    }
}
