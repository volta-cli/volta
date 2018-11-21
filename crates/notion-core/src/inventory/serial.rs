use std::collections::{BTreeSet, HashSet};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::path::Path;

use fs::read_dir_eager;
use notion_fail::{Fallible, ResultExt};
use path;
use super::{NodeCollection, YarnCollection};

use regex::Regex;
use semver::Version;

fn versions_matching(dir: &Path, re: &Regex) -> Fallible<BTreeSet<Version>> {
    Ok(read_dir_eager(dir)?
        .filter(|(_, metadata)| {
            metadata.is_file()
        })
        .filter_map(|(entry, _)| {
            if let Some(file_name) = entry.path().file_name() {
                if let Some(caps) = re.captures(&file_name.to_string_lossy()) {
                    return Some(Version::parse(&caps["version"]).unknown());
                }
            }
            None
        })
        .collect::<Fallible<BTreeSet<Version>>>()?)
}

impl NodeCollection {
    pub(crate) fn load() -> Fallible<Self> {
        let re = Regex::new(r"(?x)
            node
            -
            v(?P<version>\d+\.\d+\.\d+) # Node version
            -
            (?P<os>[a-z]+)              # operating system
            -
            (?P<arch>[a-z0-9]+)         # architecture
            \.(zip|tar\.gz)
            ").unwrap();

        let versions = versions_matching(&path::node_inventory_dir()?, &re)?;

        Ok(NodeCollection {
            versions: versions,
            phantom: PhantomData,
        })
    }
}

impl YarnCollection {
    pub(crate) fn load() -> Fallible<Self> {
        let re = Regex::new(r"(?x)
            yarn
            -
            v(?P<version>\d+\.\d+\.\d+) # Yarn version
            \.tar\.gz
            ").unwrap();

        let versions = versions_matching(&path::yarn_inventory_dir()?, &re)?;

        Ok(YarnCollection {
            versions: versions,
            phantom: PhantomData,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Index(Vec<Entry>);

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub version: String,
    pub npm: Option<String>,
    pub files: Vec<String>,
}

impl Index {
    pub fn into_index(self) -> Fallible<super::Index> {
        let mut entries = Vec::new();
        for entry in self.0 {
            if let Some(npm) = entry.npm {
                let data = super::NodeDistroFiles {
                    files: HashSet::from_iter(entry.files.into_iter()),
                };
                let mut version = &entry.version[..];
                version = version.trim();
                if version.starts_with('v') {
                    version = &version[1..];
                }
                entries.push(super::Entry {
                    version: Version::parse(version).unknown()?,
                    npm: Version::parse(&npm).unknown()?,
                    files: data
                });
            }
        }
        Ok(super::Index { entries })
    }
}
