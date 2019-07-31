use std::convert::{TryFrom, TryInto};
use std::fs::{read_to_string, write};
use std::path::PathBuf;

use super::package::{BinConfig, BinLoader};
use crate::error::ErrorDetails;
use crate::fs::ensure_containing_dir_exists;
use crate::path;
use crate::toolchain;
use crate::version::version_serde;
use semver::Version;
use serde::{Deserialize, Serialize};
use volta_fail::{Fallible, ResultExt, VoltaError};

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
        ensure_containing_dir_exists(&bin_config_path)?;
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
