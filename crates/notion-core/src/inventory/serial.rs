use std::collections::{BTreeSet, HashSet};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::path::Path;

use super::{NodeCollection, YarnCollection};
use fs::read_dir_eager;
use notion_fail::{Fallible, ResultExt};
use path;

use regex::Regex;
use semver::Version;

/// Reads the contents of a directory and returns the set of all versions found
/// in the directory's listing by matching filenames against the specified regex
/// and parsing the `version` named capture as a semantic version.
///
/// The regex should contain the `version` named capture by using the Rust regex
/// syntax `?P<version>`.
fn versions_matching(dir: &Path, re: &Regex) -> Fallible<BTreeSet<Version>> {
    Ok(read_dir_eager(dir)?
        .filter(|(_, metadata)| metadata.is_file())
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

        Ok(NodeCollection {
            versions: versions,
            phantom: PhantomData,
        })
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

        Ok(YarnCollection {
            versions: versions,
            phantom: PhantomData,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeIndex(Vec<NodeEntry>);

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeEntry {
    pub version: String,
    pub npm: Option<String>,
    pub files: Vec<String>,
}

fn trim_version(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('v') {
        s[1..].trim()
    } else {
        s
    }
}

impl NodeIndex {
    pub fn into_index(self) -> Fallible<super::NodeIndex> {
        let mut entries = Vec::new();
        for entry in self.0 {
            if let Some(npm) = entry.npm {
                let data = super::NodeDistroFiles {
                    files: HashSet::from_iter(entry.files.into_iter()),
                };
                let version = trim_version(&entry.version[..]);
                entries.push(super::NodeEntry {
                    version: Version::parse(version).unknown()?,
                    npm: Version::parse(&npm).unknown()?,
                    files: data,
                });
            }
        }
        Ok(super::NodeIndex { entries })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YarnIndex(Vec<YarnEntry>);

#[derive(Serialize, Deserialize, Debug)]
pub struct YarnEntry {
    /// Yarn releases are given a tag name of the form "v$version" where $version
    /// is the release's version string.
    pub tag_name: String,

    /// The GitHub API provides a list of assets. Some Yarn releases don't include
    /// a tarball, so we don't support them and remove them from the set of available
    /// Yarn versions.
    pub assets: Vec<YarnAsset>,
}

impl YarnEntry {
    /// Is this entry a full release, i.e., does this entry's asset list include a
    /// proper release tarball?
    fn is_full_release(&self) -> bool {
        let release_filename = &format!("yarn-{}.tar.gz", self.tag_name)[..];
        self.assets
            .iter()
            .any(|&YarnAsset { ref name }| name == release_filename)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YarnAsset {
    /// The filename of an asset included in a Yarn GitHub release.
    pub name: String,
}

impl YarnIndex {
    pub fn into_index(self) -> Fallible<super::YarnIndex> {
        let mut entries = BTreeSet::new();
        for entry in self.0 {
            if entry.is_full_release() {
                let version = trim_version(&entry.tag_name[..]);
                entries.insert(Version::parse(version).unknown()?);
            }
        }
        Ok(super::YarnIndex { entries })
    }
}
