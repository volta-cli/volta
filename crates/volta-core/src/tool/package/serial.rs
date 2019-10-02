use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};

use super::install::{BinConfig, BinLoader, PackageConfig};
use super::resolve::PackageIndex;
use super::PackageDetails;
use crate::error::ErrorDetails;
use crate::path;
use crate::toolchain;
use crate::version::version_serde;
use fs_utils::ensure_containing_dir_exists;
use semver::Version;
use serde::{Deserialize, Serialize};
use volta_fail::{Fallible, ResultExt, VoltaError};

/// Package Metadata Response
///
/// See npm registry API doc:
/// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md
#[derive(Deserialize, Debug)]
pub struct RawPackageMetadata {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, RawPackageVersionInfo>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: RawPackageDistTags,
}

#[derive(Deserialize, Debug)]
pub struct RawPackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawPackageDistTags {
    #[serde(with = "version_serde")]
    pub latest: Version,
    pub beta: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawDistInfo {
    pub shasum: String,
    pub tarball: String,
}

impl From<RawPackageMetadata> for PackageIndex {
    fn from(serial: RawPackageMetadata) -> PackageIndex {
        let mut entries: Vec<PackageDetails> = serial
            .versions
            .into_iter()
            .map(|(_, version_info)| PackageDetails {
                version: version_info.version,
                tarball_url: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            })
            .collect();

        entries.sort_by(|a, b| b.version.cmp(&a.version));

        PackageIndex {
            latest: serial.dist_tags.latest,
            entries,
        }
    }
}

// Data structures for `npm view` data
//
// $ npm view --json gulp@latest
// {
//   "name": "gulp",
//   "description": "The streaming build system.",
//   "dist-tags": {
//     "latest": "4.0.2"
//   },
//   "version": "4.0.2",
//   "engines": {
//     "node": ">= 0.10"
//   },
//   "dist": {
//     "shasum": "543651070fd0f6ab0a0650c6a3e6ff5a7cb09caa",
//     "tarball": "https://registry.npmjs.org/gulp/-/gulp-4.0.2.tgz",
//   },
//   (...and lots of other stuff we don't use...)
// }
//
#[derive(Deserialize, Clone, Debug)]
pub struct NpmViewData {
    pub name: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
    #[serde(rename = "dist-tags")]
    pub dist_tags: RawPackageDistTags,
}

impl From<NpmViewData> for PackageDetails {
    fn from(view_data: NpmViewData) -> PackageDetails {
        PackageDetails {
            version: view_data.version,
            tarball_url: view_data.dist.tarball,
            shasum: view_data.dist.shasum,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawPackageConfig {
    pub name: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub platform: toolchain::serial::Platform,
    pub bins: Vec<String>,
}

impl RawPackageConfig {
    pub fn to_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self)
            .with_context(|_| ErrorDetails::StringifyPackageConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|_| ErrorDetails::ParsePackageConfigError)
    }

    // Write the package config info to disk
    pub fn write(self) -> Fallible<()> {
        let config_file_path = path::user_package_config_file(&self.name)?;
        let src = self.to_json()?;
        ensure_containing_dir_exists(&config_file_path).with_context(|_| {
            ErrorDetails::ContainingDirError {
                path: config_file_path.clone(),
            }
        })?;
        write(&config_file_path, src).with_context(|_| ErrorDetails::WritePackageConfigError {
            file: config_file_path,
        })
    }
}

impl TryFrom<RawPackageConfig> for PackageConfig {
    type Error = VoltaError;

    fn try_from(raw: RawPackageConfig) -> Fallible<PackageConfig> {
        let platform = raw
            .platform
            .into_platform()?
            .ok_or(ErrorDetails::NoBinPlatform {
                binary: raw.name.clone(),
            })?;
        Ok(PackageConfig {
            name: raw.name,
            version: raw.version,
            platform,
            bins: raw.bins,
        })
    }
}

impl PackageConfig {
    pub fn from_file(file: &Path) -> Fallible<Self> {
        let config_src =
            read_to_string(file).with_context(|_| ErrorDetails::ReadPackageConfigError {
                file: file.to_path_buf(),
            })?;
        RawPackageConfig::from_json(config_src)?.try_into()
    }
}

impl From<PackageConfig> for RawPackageConfig {
    fn from(full: PackageConfig) -> RawPackageConfig {
        RawPackageConfig {
            name: full.name,
            version: full.version,
            platform: full.platform.to_serial(),
            bins: full.bins,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawBinConfig {
    pub name: String,
    pub package: String,
    #[serde(with = "version_serde")]
    pub version: Version,
    pub path: String,
    pub platform: toolchain::serial::Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader: Option<RawBinLoader>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawBinLoader {
    pub command: String,
    pub args: Vec<String>,
}

impl RawBinConfig {
    pub fn to_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|_| ErrorDetails::StringifyBinConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|_| ErrorDetails::ParseBinConfigError)
    }

    /// Write the config to disk
    pub fn write(self) -> Fallible<()> {
        let bin_config_path = path::user_tool_bin_config(&self.name)?;
        let src = self.to_json()?;
        ensure_containing_dir_exists(&bin_config_path).with_context(|_| {
            ErrorDetails::ContainingDirError {
                path: bin_config_path.clone(),
            }
        })?;
        write(&bin_config_path, src).with_context(|_| ErrorDetails::WriteBinConfigError {
            file: bin_config_path,
        })
    }
}

impl BinConfig {
    pub fn from_file(file: PathBuf) -> Fallible<Self> {
        let config_src =
            read_to_string(&file).with_context(|_| ErrorDetails::ReadBinConfigError { file })?;
        RawBinConfig::from_json(config_src)?.try_into()
    }
}

impl TryFrom<RawBinConfig> for BinConfig {
    type Error = VoltaError;

    fn try_from(raw: RawBinConfig) -> Fallible<BinConfig> {
        let platform = raw
            .platform
            .into_platform()?
            .ok_or(ErrorDetails::NoBinPlatform {
                binary: raw.name.clone(),
            })?;
        Ok(BinConfig {
            name: raw.name,
            package: raw.package,
            version: raw.version,
            path: raw.path,
            platform,
            loader: raw.loader.map(|l| l.into()),
        })
    }
}

impl From<BinConfig> for RawBinConfig {
    fn from(full: BinConfig) -> RawBinConfig {
        RawBinConfig {
            name: full.name,
            package: full.package,
            version: full.version,
            path: full.path,
            platform: full.platform.to_serial(),
            loader: full.loader.map(Into::into),
        }
    }
}

impl From<RawBinLoader> for BinLoader {
    fn from(raw: RawBinLoader) -> BinLoader {
        BinLoader {
            command: raw.command,
            args: raw.args,
        }
    }
}

impl From<BinLoader> for RawBinLoader {
    fn from(full: BinLoader) -> RawBinLoader {
        RawBinLoader {
            command: full.command,
            args: full.args,
        }
    }
}

// Data structures for `npm pack` data
//
// $ npm pack --dry-run --json ember-cli
// {
//   "id": "ember-cli@3.13.1",
//   "name": "ember-cli",
//   "version": "3.13.1",
//   "from": "ember-cli@latest",
//   "size": 199017,
//   "unpackedSize": 827812,
//   "shasum": "8daefb108130740cd79ad7e4e1c9138fb1f7313d",
//   "integrity": "sha512-CMVLpJYseyCNmN2Tp3vTmTFTXPSZlMQB7q2uoZ+ZTKMgdQ4ekeceW9mVAC4XwXm2FW+v8liowP+co/Bu1xUbPg==",
//   "filename": "ember-cli-3.13.1.tgz",
//   "files": [
//     {
//       "path": "blueprints/app/files/.editorconfig",
//       "size": 368,
//       "mode": 420
//     },
//     (and lots more files...)
//    ],
//   "entryCount": 290,
//   "bundled": []
// }
//
#[derive(Deserialize, Clone, Debug)]
pub struct NpmPackData {
    // there's a lot more in there, but right now just care about the filename
    pub filename: String,
}
