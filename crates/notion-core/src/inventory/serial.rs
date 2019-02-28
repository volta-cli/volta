use std::collections::HashMap;
use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

use super::{NodeCollection, PackageCollection, YarnCollection};
use crate::distro::package;
use crate::fs::ensure_containing_dir_exists;
use crate::fs::read_dir_eager;
use crate::path;
use crate::toolchain;
use crate::version::version_parse_error;
use notion_fail::{Fallible, ResultExt};
use readext::ReadExt;

use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};

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

impl PackageCollection {
    pub(crate) fn load() -> Fallible<Self> {
        // TODO: all of this mess
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

        Ok(PackageCollection {
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
                    version: Version::parse(version).with_context(version_parse_error)?,
                    npm: Version::parse(&npm).with_context(version_parse_error)?,
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

// see npm registry API doc:
// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetadata {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, PackageVersionInfo>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: PackageDistTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    pub version: String,
    pub dist: DistInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDistTags {
    pub latest: String,
    pub beta: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DistInfo {
    pub shasum: String,
    pub tarball: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub platform: toolchain::serial::Platform,
    pub bins: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BinConfig {
    pub name: String,
    pub package: String,
    pub version: String,
    pub path: String,
    pub platform: toolchain::serial::Platform,
}

impl PackageMetadata {
    pub fn into_index(self) -> Fallible<package::PackageIndex> {
        let latest_version = Version::parse(&self.dist_tags.latest).unknown()?;

        let mut entries = Vec::new();
        for (_, version_info) in self.versions {
            let parsed_version = Version::parse(&version_info.version).unknown()?;
            let entry = package::PackageEntry {
                version: parsed_version,
                tarball: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            };
            entries.push(entry);
        }

        // sort entries by version, largest to smallest
        entries.sort_by(|a, b| a.version.cmp(&b.version).reverse());

        Ok(package::PackageIndex {
            latest: latest_version,
            entries: entries,
        })
    }
}

impl package::PackageConfig {
    pub fn to_serial(&self) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.to_string(),
            platform: self.platform.to_serial(),
            bins: self.bins.clone(),
        }
    }
}

impl package::BinConfig {
    pub fn from_file(file: PathBuf) -> Fallible<Self> {
        let config_src = File::open(file).unknown()?.read_into_string().unknown()?;
        BinConfig::from_json(config_src.to_string())?.into_config()
    }

    pub fn to_serial(&self) -> BinConfig {
        BinConfig {
            name: self.name.to_string(),
            package: self.package.to_string(),
            version: self.version.to_string(),
            path: self.path.to_string(),
            platform: self.platform.to_serial(),
        }
    }
}

impl PackageConfig {
    pub fn to_json(&self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).unknown()
    }

    // not used yet - needed for listing and uninstall
    #[allow(dead_code)]
    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).unknown()
    }

    // write the package config info to disk
    pub fn write(&self) -> Fallible<()> {
        let src = self.to_json()?;
        let config_file_path = path::user_package_config_file(&self.name)?;
        ensure_containing_dir_exists(&config_file_path)?;
        let mut file = File::create(&config_file_path).unknown()?;
        file.write_all(src.as_bytes()).unknown()?;
        Ok(())
    }
}

impl BinConfig {
    pub fn to_json(&self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).unknown()
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).unknown()
    }

    // write the binary config info to disk
    pub fn write(&self) -> Fallible<()> {
        let src = self.to_json()?;
        let bin_config_path = path::user_tool_bin_config(&self.name)?;
        ensure_containing_dir_exists(&bin_config_path)?;
        let mut file = File::create(&bin_config_path).unknown()?;
        file.write_all(src.as_bytes()).unknown()?;
        Ok(())
    }

    pub fn into_config(self) -> Fallible<package::BinConfig> {
        Ok(package::BinConfig {
            name: self.name,
            package: self.package,
            version: Version::parse(&self.version).unknown()?,
            path: self.path,
            platform: self.platform.into_image()?.unwrap(),
        })
    }
}
