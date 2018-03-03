use super::super::catalog;

use std::collections::{HashSet, BTreeMap};
use std::iter::FromIterator;

use semver::Version;
use error::{Fallible, ResultExt};

#[derive(Serialize, Deserialize)]
pub struct Index(Vec<Entry>);

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub version: String,
    pub files: Vec<String>
}

impl Index {
    pub fn into_index(self) -> Fallible<catalog::Index> {
        let mut entries = BTreeMap::new();
        for entry in self.0 {
            let data = catalog::VersionData {
                files: HashSet::from_iter(entry.files.into_iter())
            };
            let mut version = &entry.version[..];
            version = version.trim();
            if version.starts_with('v') {
                version = &version[1..];
            }
            entries.insert(Version::parse(version).unknown()?, data);
        }
        Ok(catalog::Index { entries })
    }
}
