use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::{read_to_string, write};
use std::io;
use std::path::{Path, PathBuf};

use super::super::registry::RawDistInfo;
use super::PackageDetails;
use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::layout::volta_home;
use crate::platform::PlatformSpec;
use crate::toolchain;
use crate::version::{hashmap_version_serde, version_serde};
use fs_utils::ensure_containing_dir_exists;
use semver::Version;
use serde::{Deserialize, Serialize};

/// Configuration information about an installed package.
///
/// This information will be stored in ~/.volta/tools/user/packages/<package>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "version": "1.4.0",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null
///   },
///   "bins": [
///     "cowsay",
///     "cowthink"
///   ]
/// }
#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct PackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    pub version: Version,
    /// The platform used to install this package
    pub platform: PlatformSpec,
    /// The binaries installed by this package
    pub bins: Vec<String>,
}

/// Configuration information about an installed binary from a package.
///
/// This information will be stored in ~/.volta/tools/user/bins/<bin-name>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "package": "cowsay",
///   "version": "1.4.0",
///   "path": "./cli.js",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null,
///     "loader": {
///       "exe": "node",
///       "args": []
///     }
///   }
/// }
pub struct BinConfig {
    /// The binary name
    pub name: String,
    /// The package that installed this binary
    pub package: String,
    /// The package version
    pub version: Version,
    /// The relative path of the binary in the installed package
    pub path: String,
    /// The platform used to install this binary
    pub platform: PlatformSpec,
    /// The loader information for the script, if any
    pub loader: Option<BinLoader>,
}

/// Information about the Shebang script loader (e.g. `#!/usr/bin/env node`)
///
/// Only important for Windows at the moment, as Windows does not natively understand script
/// loaders, so we need to provide that behavior when calling a script that uses one
pub struct BinLoader {
    /// The command used to run a script
    pub command: String,
    /// Any additional arguments specified for the loader
    pub args: Vec<String>,
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
    #[serde(
        rename = "dist-tags",
        deserialize_with = "hashmap_version_serde::deserialize"
    )]
    pub dist_tags: HashMap<String, Version>,
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
    pub fn into_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|| ErrorKind::StringifyPackageConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|| ErrorKind::ParsePackageConfigError)
    }

    // Write the package config info to disk
    pub fn write(self) -> Fallible<()> {
        let config_file_path = volta_home()?.default_package_config_file(&self.name);
        let src = self.into_json()?;
        ensure_containing_dir_exists(&config_file_path).with_context(|| {
            ErrorKind::ContainingDirError {
                path: config_file_path.clone(),
            }
        })?;
        write(&config_file_path, src).with_context(|| ErrorKind::WritePackageConfigError {
            file: config_file_path,
        })
    }
}

impl TryFrom<RawPackageConfig> for PackageConfig {
    type Error = VoltaError;

    fn try_from(raw: RawPackageConfig) -> Fallible<PackageConfig> {
        let platform = raw
            .platform
            .into_platform()
            .ok_or(ErrorKind::NoBinPlatform {
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
            read_to_string(file).with_context(|| ErrorKind::ReadPackageConfigError {
                file: file.to_path_buf(),
            })?;
        RawPackageConfig::from_json(config_src)?.try_into()
    }

    pub fn from_file_if_exists(file: &Path) -> Fallible<Option<Self>> {
        match read_to_string(file) {
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(VoltaError::from_source(
                        error,
                        ErrorKind::ReadPackageConfigError {
                            file: file.to_path_buf(),
                        },
                    ))
                }
            }
            Ok(config_src) => RawPackageConfig::from_json(config_src)?
                .try_into()
                .map(Some),
        }
    }
}

impl From<PackageConfig> for RawPackageConfig {
    fn from(full: PackageConfig) -> RawPackageConfig {
        RawPackageConfig {
            name: full.name,
            version: full.version,
            platform: toolchain::serial::Platform::of(&full.platform),
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
    pub fn into_json(self) -> Fallible<String> {
        serde_json::to_string_pretty(&self).with_context(|| ErrorKind::StringifyBinConfigError)
    }

    pub fn from_json(src: String) -> Fallible<Self> {
        serde_json::de::from_str(&src).with_context(|| ErrorKind::ParseBinConfigError)
    }

    /// Write the config to disk
    pub fn write(self) -> Fallible<()> {
        let bin_config_path = volta_home()?.default_tool_bin_config(&self.name);
        let src = self.into_json()?;
        ensure_containing_dir_exists(&bin_config_path).with_context(|| {
            ErrorKind::ContainingDirError {
                path: bin_config_path.clone(),
            }
        })?;
        write(&bin_config_path, src).with_context(|| ErrorKind::WriteBinConfigError {
            file: bin_config_path,
        })
    }
}

impl BinConfig {
    pub fn from_file(file: PathBuf) -> Fallible<Self> {
        let config_src =
            read_to_string(&file).with_context(|| ErrorKind::ReadBinConfigError { file })?;
        RawBinConfig::from_json(config_src)?.try_into()
    }

    pub fn from_file_if_exists(file: PathBuf) -> Fallible<Option<Self>> {
        match read_to_string(&file) {
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(VoltaError::from_source(
                        error,
                        ErrorKind::ReadBinConfigError { file },
                    ))
                }
            }
            Ok(config_src) => RawBinConfig::from_json(config_src)?
                .try_into()
                .map(|config| Some(config)),
        }
    }
}

impl TryFrom<RawBinConfig> for BinConfig {
    type Error = VoltaError;

    fn try_from(raw: RawBinConfig) -> Fallible<BinConfig> {
        let platform = raw
            .platform
            .into_platform()
            .ok_or(ErrorKind::NoBinPlatform {
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
            platform: toolchain::serial::Platform::of(&full.platform),
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
