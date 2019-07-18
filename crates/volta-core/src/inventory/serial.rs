use std::collections::BTreeSet;
use std::fs::{read_to_string, write};
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
use crate::version::{version_serde, VersionSpec};
use volta_fail::{Fallible, ResultExt};

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
    pub command: String,
    pub args: Vec<String>,
}

impl package::PackageConfig {
    pub fn from_file(file: &PathBuf) -> Fallible<Self> {
        let config_src =
            read_to_string(file).with_context(|_| ErrorDetails::ReadPackageConfigError {
                file: file.to_path_buf(),
            })?;
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
        let config_src =
            read_to_string(&file).with_context(|_| ErrorDetails::ReadBinConfigError { file })?;
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
            command: self.command.clone(),
            args: self.args.clone(),
        }
    }
}

impl PackageConfig {
    pub fn to_json(&self) -> Fallible<String> {
        serde_json::to_string_pretty(&self)
            .with_context(|_| ErrorDetails::StringifyPackageConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|_| ErrorDetails::ParsePackageConfigError)
    }

    // write the package config info to disk
    pub fn write(&self) -> Fallible<()> {
        let src = self.to_json()?;
        let config_file_path = path::user_package_config_file(&self.name)?;
        ensure_containing_dir_exists(&config_file_path)?;
        write(&config_file_path, src).with_context(|_| ErrorDetails::WritePackageConfigError {
            file: config_file_path,
        })
    }

    pub fn into_config(self) -> Fallible<package::PackageConfig> {
        Ok(package::PackageConfig {
            name: self.name.clone(),
            version: self.version,
            platform: self
                .platform
                .into_platform()?
                .ok_or(ErrorDetails::NoBinPlatform { binary: self.name })?,
            bins: self.bins,
        })
    }
}

impl BinConfig {
    pub fn to_json(&self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|_| ErrorDetails::StringifyBinConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|_| ErrorDetails::ParseBinConfigError)
    }

    // write the binary config info to disk
    pub fn write(&self) -> Fallible<()> {
        let src = self.to_json()?;
        let bin_config_path = path::user_tool_bin_config(&self.name)?;
        ensure_containing_dir_exists(&bin_config_path)?;
        write(&bin_config_path, src).with_context(|_| ErrorDetails::WriteBinConfigError {
            file: bin_config_path,
        })
    }

    pub fn into_config(self) -> Fallible<package::BinConfig> {
        Ok(package::BinConfig {
            name: self.name.clone(),
            package: self.package,
            version: self.version,
            path: self.path,
            platform: self
                .platform
                .into_platform()?
                .ok_or(ErrorDetails::NoBinPlatform { binary: self.name })?,
            loader: self.loader.map(|l| l.into_loader()),
        })
    }
}

impl BinLoader {
    pub fn into_loader(self) -> package::BinLoader {
        package::BinLoader {
            command: self.command,
            args: self.args,
        }
    }
}
