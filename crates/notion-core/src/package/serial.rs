// Serialization for npm package information

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::fs::ensure_containing_dir_exists;
use crate::path;
use crate::toolchain;
use notion_fail::Fallible;
use notion_fail::ResultExt;
use semver::Version;
use serde::{Deserialize, Serialize};


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
    pub fn into_index(self) -> Fallible<super::PackageIndex> {
        let latest_version = Version::parse(&self.dist_tags.latest).unknown()?;

        let mut entries = Vec::new();
        for (_, version_info) in self.versions {
            let parsed_version = Version::parse(&version_info.version).unknown()?;
            let entry = super::PackageEntry {
                version: parsed_version,
                tarball: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            };
            entries.push(entry);
        }

        // sort entries by version, largest to smallest
        entries.sort_by(|a, b| a.version.cmp(&b.version).reverse());

        Ok(super::PackageIndex{ latest: latest_version, entries: entries })
    }
}

impl super::PackageConfig {
    pub fn to_serial(&self) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.to_string(),
            platform: self.platform.to_serial(),
            bins: self.bins.clone(),
        }
    }
}

impl super::BinConfig {
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

    pub fn into_config(self) -> Fallible<super::BinConfig> {
        Ok(super::BinConfig {
            name: self.name,
            package: self.package,
            version: Version::parse(&self.version).unknown()?,
            path: self.path,
            platform: self.platform.into_image()?.unwrap(),
        })
    }
}
