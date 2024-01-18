use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use super::manager::PackageManager;
use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::layout::volta_home;
use crate::platform::PlatformSpec;
use crate::version::{option_version_serde, version_serde};
use fs_utils::ensure_containing_dir_exists;
use node_semver::Version;

/// Configuration information about an installed package
///
/// Will be stored in `<VOLTA_HOME>/tools/user/packages/<package>.json`
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
    /// The package manager that was used to install this package
    pub manager: PackageManager,
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

    pub fn from_file_if_exists<P>(file: P) -> Fallible<Option<Self>>
    where
        P: AsRef<Path>,
    {
        match File::open(&file) {
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(VoltaError::from_source(
                        error,
                        ErrorKind::ReadPackageConfigError {
                            file: file.as_ref().to_owned(),
                        },
                    ))
                }
            }
            Ok(config) => serde_json::from_reader(config)
                .with_context(|| ErrorKind::ParsePackageConfigError)
                .map(Some),
        }
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
    /// The package manager used to install this binary
    pub manager: PackageManager,
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

    pub fn from_file_if_exists<P>(file: P) -> Fallible<Option<Self>>
    where
        P: AsRef<Path>,
    {
        match File::open(&file) {
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(VoltaError::from_source(
                        error,
                        ErrorKind::ReadBinConfigError {
                            file: file.as_ref().to_owned(),
                        },
                    ))
                }
            }
            Ok(config) => serde_json::from_reader(config)
                .with_context(|| ErrorKind::ParseBinConfigError)
                .map(Some),
        }
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
    // The magic:
    // `serde(default)` to assign the pnpm field with a default value, this
    // ensures a seamless migration is performed from the previous package
    // platformspec which did not have a pnpm field despite the same layout.v3
    #[serde(default)]
    #[serde(with = "option_version_serde")]
    pnpm: Option<Version>,
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
            File::open(package_file).with_context(|| ErrorKind::PackageManifestReadError {
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
        // Note: For a scoped package, we should remove the scope and only use the package name
        if manifest.bin == [""] {
            manifest.bin.pop();
            manifest.bin.push(default_binary_name(&manifest.name));
        }

        Ok(manifest)
    }
}

#[derive(serde::Deserialize)]
/// Struct to read the `dependencies` out of Yarn's global manifest.
///
/// For global installs, yarn creates a `package.json` file in the `global-folder` and installs
/// global packages as dependencies of that pseudo-package
pub(super) struct GlobalYarnManifest {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
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
                // Bin names that include path separators are invalid, as they would then point to
                // other locations on the filesystem. To match the behavior of npm & Yarn, we
                // filter those values out of the list of bins.
                if !name.contains(&['/', '\\'][..]) {
                    bins.push(name);
                }
            }
            Ok(bins)
        }
    }
}

/// Determine the default binary name from the package name
///
/// For non-scoped packages, this is just the package name
/// For scoped packages, to match the behavior of the package managers, we remove the scope and use
/// only the package part, e.g. `@microsoft/rush` would have a default name of `rush`
fn default_binary_name(package_name: &str) -> String {
    if package_name.starts_with('@') {
        let mut chars = package_name.chars();

        loop {
            match chars.next() {
                Some('/') | None => break,
                _ => {}
            }
        }

        let name = chars.as_str();
        if name.is_empty() {
            package_name.to_string()
        } else {
            name.to_string()
        }
    } else {
        package_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::default_binary_name;

    #[test]
    fn default_binary_uses_full_name_if_unscoped() {
        assert_eq!(default_binary_name("my-package"), "my-package");
    }

    #[test]
    fn default_binary_removes_scope() {
        assert_eq!(default_binary_name("@scope/my-package"), "my-package");
    }
}
