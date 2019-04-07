use std::collections::HashMap;
use std::collections::{BTreeSet, HashSet};
use std::fs::{read_to_string, write};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

use super::{NodeCollection, PackageCollection, YarnCollection};
use crate::distro::package;
use crate::error::ErrorDetails;
use crate::fs::ensure_containing_dir_exists;
use crate::fs::read_dir_eager;
use crate::path;
use crate::toolchain;
use crate::version::{option_version_serde, version_serde};
use notion_fail::{throw, Fallible, ResultExt};

use regex::Regex;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};

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

fn lts_version_serde<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(_t) => Ok(true),
        Err(_e) => Ok(false),
    }
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
    // loads an empty PackageCollection
    // ISSUE(#288) Collection only supports versions - for packages we also need names
    pub(crate) fn load() -> Fallible<Self> {
        Ok(PackageCollection {
            versions: BTreeSet::new(),
            phantom: PhantomData,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeIndex(Vec<NodeEntry>);

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeEntry {
    #[serde(with = "version_serde")]
    pub version: Version,
    #[serde(default)] // handles Option
    #[serde(with = "option_version_serde")]
    pub npm: Option<Version>,
    pub files: Vec<String>,
    #[serde(deserialize_with = "lts_version_serde")]
    pub lts: bool,
}

impl NodeIndex {
    pub fn into_index(self) -> Fallible<super::NodeIndex> {
        let mut entries = Vec::new();
        for entry in self.0 {
            if let Some(npm) = entry.npm {
                let data = super::NodeDistroFiles {
                    files: HashSet::from_iter(entry.files.into_iter()),
                };
                entries.push(super::NodeEntry {
                    version: entry.version,
                    npm,
                    files: data,
                    lts: entry.lts,
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
    #[serde(with = "version_serde")]
    pub tag_name: Version,

    /// The GitHub API provides a list of assets. Some Yarn releases don't include
    /// a tarball, so we don't support them and remove them from the set of available
    /// Yarn versions.
    pub assets: Vec<YarnAsset>,
}

impl YarnEntry {
    /// Is this entry a full release, i.e., does this entry's asset list include a
    /// proper release tarball?
    fn is_full_release(&self) -> bool {
        let release_filename = &format!("yarn-v{}.tar.gz", self.tag_name)[..];
        println!("checking release filename: {}", release_filename);
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
                entries.insert(entry.tag_name);
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
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: DistInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDistTags {
    #[serde(with = "version_serde")]
    pub latest: Version,
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
    #[serde(with = "version_serde")]
    pub version: Version,
    pub platform: toolchain::serial::Platform,
    pub bins: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BinConfig {
    pub name: String,
    pub package: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub path: String,
    pub platform: toolchain::serial::Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader: Option<BinLoader>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BinLoader {
    pub exe: String,
    pub args: Vec<String>,
}

impl PackageMetadata {
    pub fn into_index(self) -> package::PackageIndex {
        let mut entries = Vec::new();
        for (_, version_info) in self.versions {
            let entry = package::PackageEntry {
                version: version_info.version,
                tarball: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            };
            entries.push(entry);
        }

        // sort entries by version, largest to smallest
        entries.sort_by(|a, b| b.version.cmp(&a.version));

        package::PackageIndex {
            latest: self.dist_tags.latest,
            entries: entries,
        }
    }
}

impl package::PackageConfig {
    pub fn from_file(file: &PathBuf) -> Fallible<Self> {
        if !file.exists() {
            throw!(ErrorDetails::PackageConfigNotFound);
        }
        let config_src = read_to_string(file).unknown()?;
        PackageConfig::from_json(config_src)?.into_config()
    }

    pub fn to_serial(&self) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.clone(),
            platform: self.platform.to_serial(),
            bins: self.bins.clone(),
        }
    }
}

impl package::BinConfig {
    pub fn from_file(file: PathBuf) -> Fallible<Self> {
        let config_src = read_to_string(file).unknown()?;
        BinConfig::from_json(config_src)?.into_config()
    }

    pub fn to_serial(&self) -> BinConfig {
        BinConfig {
            name: self.name.to_string(),
            package: self.package.to_string(),
            version: self.version.clone(),
            path: self.path.to_string(),
            platform: self.platform.to_serial(),
            loader: self.loader.as_ref().map(|l| l.to_serial()),
        }
    }
}

impl package::BinLoader {
    pub fn to_serial(&self) -> BinLoader {
        BinLoader {
            exe: self.exe.clone(),
            args: self.args.clone(),
        }
    }
}

impl PackageConfig {
    pub fn to_json(&self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).unknown()
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).unknown()
    }

    // write the package config info to disk
    pub fn write(&self) -> Fallible<()> {
        let src = self.to_json()?;
        let config_file_path = path::user_package_config_file(&self.name)?;
        ensure_containing_dir_exists(&config_file_path)?;
        write(config_file_path, src).unknown()
    }

    pub fn into_config(self) -> Fallible<package::PackageConfig> {
        Ok(package::PackageConfig {
            name: self.name.clone(),
            version: self.version,
            platform: self
                .platform
                .into_image()?
                .ok_or(ErrorDetails::NoBinPlatform { binary: self.name })?,
            bins: self.bins,
        })
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
        write(bin_config_path, src).unknown()
    }

    pub fn into_config(self) -> Fallible<package::BinConfig> {
        Ok(package::BinConfig {
            name: self.name.clone(),
            package: self.package,
            version: self.version,
            path: self.path,
            platform: self
                .platform
                .into_image()?
                .ok_or(ErrorDetails::NoBinPlatform { binary: self.name })?,
            loader: self.loader.map(|l| l.into_loader()),
        })
    }
}

impl BinLoader {
    pub fn into_loader(self) -> package::BinLoader {
        package::BinLoader {
            exe: self.exe,
            args: self.args,
        }
    }
}
