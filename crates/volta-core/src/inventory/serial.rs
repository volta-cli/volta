use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::{NodeCollection, PackageCollection, YarnCollection};
use crate::error::ErrorDetails;
use crate::fs::read_dir_eager;
use crate::path;
use crate::version::VersionSpec;
use volta_fail::{Fallible, ResultExt};

use regex::Regex;
use semver::Version;

/// Reads the contents of a directory and returns the set of all versions found
/// in the directory's listing by matching filenames against the specified regex
/// and parsing the `version` named capture as a semantic version.
///
/// The regex should contain the `version` named capture by using the Rust regex
/// syntax `?P<version>`.
fn versions_matching(dir: &Path, re: &Regex) -> Fallible<BTreeSet<Version>> {
    let contents = read_dir_eager(dir).with_context(|_| ErrorDetails::ReadInventoryDirError {
        dir: dir.to_path_buf(),
    })?;
    contents
        .filter(|(_, metadata)| metadata.is_file())
        .filter_map(|(entry, _)| {
            if let Some(file_name) = entry.path().file_name() {
                if let Some(caps) = re.captures(&file_name.to_string_lossy()) {
                    return Some(VersionSpec::parse_version(&caps["version"]));
                }
            }
            None
        })
        .collect::<Fallible<BTreeSet<Version>>>()
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

        Ok(YarnCollection { versions })
    }
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
