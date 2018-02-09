use super::super::session;

use std::collections::{HashSet, BTreeMap};
use std::iter::FromIterator;

use semver::Version;
use failure;

#[derive(Serialize, Deserialize)]
pub struct Index(Vec<Entry>);

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub version: String,
    pub files: Vec<String>
}

impl Index {
    pub fn into_index(self) -> Result<session::Index, failure::Error> {
        let mut entries = BTreeMap::new();
        for entry in self.0 {
            let data = session::VersionData {
                files: HashSet::from_iter(entry.files.into_iter())
            };
            let mut version = &entry.version[..];
            version = version.trim();
            if version.starts_with('v') {
                version = &version[1..];
            }
            entries.insert(Version::parse(version)?, data);
        }
        Ok(session::Index { entries })
    }
}
