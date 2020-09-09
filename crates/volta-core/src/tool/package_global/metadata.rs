use std::fs::File;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::platform::PlatformSpec;
use crate::version::{option_version_serde, version_serde};
use fs_utils::ensure_containing_dir_exists;
use semver::Version;

/// Configuration information about an installed package
///
/// Will be stored in <VOLTA_HOME>/tools/user/packages/<package>.json
#[derive(serde::Serialize, serde::Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct PackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// The platform used to install this package
    #[serde(with = "RawPlatformSpec")]
    pub platform: PlatformSpec,
    /// The binaries installed by this package
    pub bins: Vec<String>,
}

impl PackageConfig {
    /// Parse a `PackageConfig` instance from a config file
    pub fn from_file<P>(file: P) -> Fallible<Self>
    where
        P: AsRef<Path>,
    {
        let config = File::open(&file).with_context(|| ErrorKind::ReadPackageConfigError {
            file: file.as_ref().to_owned(),
        })?;
        serde_json::from_reader(config).with_context(|| ErrorKind::ParsePackageConfigError)
    }

    /// Write this `PackageConfig` into the appropriate config file
    pub fn write(self) -> Fallible<()> {
        let config_file_path = volta_home()?.default_package_config_file(&self.name);

        ensure_containing_dir_exists(&config_file_path).with_context(|| {
            ErrorKind::ContainingDirError {
                path: config_file_path.clone(),
            }
        })?;

        let file = File::create(&config_file_path).with_context(|| {
            ErrorKind::WritePackageConfigError {
                file: config_file_path,
            }
        })?;
        serde_json::to_writer_pretty(file, &self)
            .with_context(|| ErrorKind::StringifyPackageConfigError)
    }
}

/// Configuration information about a single installed binary from a package
///
/// Will be stored in <VOLTA_HOME>/tools/user/bins/<bin-name>.json
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BinConfig {
    /// The binary name
    pub name: String,
    /// The package that installed the binary
    pub package: String,
    /// The package version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// The platform used to install this binary
    #[serde(with = "RawPlatformSpec")]
    pub platform: PlatformSpec,
}

impl BinConfig {
    /// Parse a `BinConfig` instance from the given config file
    pub fn from_file<P>(file: P) -> Fallible<Self>
    where
        P: AsRef<Path>,
    {
        let config = File::open(&file).with_context(|| ErrorKind::ReadBinConfigError {
            file: file.as_ref().to_owned(),
        })?;
        serde_json::from_reader(config).with_context(|| ErrorKind::ParseBinConfigError)
    }

    /// Write this `BinConfig` to the appropriate config file
    pub fn write(self) -> Fallible<()> {
        let config_file_path = volta_home()?.default_tool_bin_config(&self.name);

        ensure_containing_dir_exists(&config_file_path).with_context(|| {
            ErrorKind::ContainingDirError {
                path: config_file_path.clone(),
            }
        })?;

        let file =
            File::create(&config_file_path).with_context(|| ErrorKind::WriteBinConfigError {
                file: config_file_path,
            })?;
        serde_json::to_writer_pretty(file, &self)
            .with_context(|| ErrorKind::StringifyBinConfigError)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "PlatformSpec")]
struct RawPlatformSpec {
    #[serde(with = "version_serde")]
    node: Version,
    #[serde(with = "option_version_serde")]
    npm: Option<Version>,
    #[serde(with = "option_version_serde")]
    yarn: Option<Version>,
}

/// The relevant information we need out of a package's `package.json` file
///
/// This includes the exact Version (since we can install using a range)
/// and the list of bins provided by the package.
#[derive(serde::Deserialize)]
pub struct PackageManifest {
    /// The name of the package
    pub name: String,
    /// The version of the package
    #[serde(deserialize_with = "version_serde::deserialize")]
    pub version: Version,
    /// The `bin` section, containing a map of binary names to locations
    #[serde(default, deserialize_with = "serde_bins::deserialize")]
    pub bin: Vec<String>,
}

impl PackageManifest {
    /// Parse the `package.json` for a given package directory
    pub fn for_dir(package: &str, package_root: &Path) -> Fallible<Self> {
        let package_file = package_root.join("package.json");
        let file =
            File::open(&package_file).with_context(|| ErrorKind::PackageManifestReadError {
                package: package.into(),
            })?;

        let mut manifest: Self = serde_json::de::from_reader(file).with_context(|| {
            ErrorKind::PackageManifestParseError {
                package: package.into(),
            }
        })?;

        // If the bin list contains only an empty string, that means `bin` was a string value,
        // rather than a map. In that case, to match `npm`s behavior, we use the name of the package
        // as the bin name.
        if manifest.bin == [""] {
            manifest.bin.pop();
            manifest.bin.push(manifest.name.clone());
        }

        Ok(manifest)
    }
}

mod serde_bins {
    use std::fmt;

    use serde::de::{Deserializer, Error, MapAccess, Visitor};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BinMapVisitor)
    }

    struct BinMapVisitor;

    impl<'de> Visitor<'de> for BinMapVisitor {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("string or map")
        }

        // Handle String values with only the path
        fn visit_str<E>(self, _path: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            // Use an empty string as a placeholder for the binary name, since at this level we
            // don't know the binary name for sure (npm uses the package name in this case)
            Ok(vec![String::new()])
        }

        // Handle maps of Name -> Path
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut bins = Vec::new();
            while let Some((name, _)) = access.next_entry::<String, String>()? {
                bins.push(name);
            }
            Ok(bins)
        }
    }
}
