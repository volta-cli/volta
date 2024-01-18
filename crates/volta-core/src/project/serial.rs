use std::collections::HashMap;
use std::fmt;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::PartialPlatform;
use crate::error::{Context, ErrorKind, Fallible};
use crate::version::parse_version;
use dunce::canonicalize;
use node_semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type DependencyMapIterator = std::iter::Chain<
    std::option::IntoIter<HashMap<String, String>>,
    std::option::IntoIter<HashMap<String, String>>,
>;

pub(super) struct Manifest {
    pub dependency_maps: DependencyMapIterator,
    pub platform: Option<PartialPlatform>,
    pub extends: Option<PathBuf>,
}

impl Manifest {
    pub fn from_file(file: &Path) -> Fallible<Self> {
        let raw = RawManifest::from_file(file)?;

        let dependency_maps = raw.dependencies.into_iter().chain(raw.dev_dependencies);

        let (platform, extends) = match raw.volta {
            Some(toolchain) => {
                let (partial, extends) = toolchain.parse_split()?;

                let next = extends
                    .map(|path| {
                        // Invariant: Since we successfully parsed it, we know we have a path to a file
                        let unresolved = file
                            .parent()
                            .expect("File paths always have a parent")
                            .join(&path);
                        canonicalize(unresolved)
                            .with_context(|| ErrorKind::ExtensionPathError { path })
                    })
                    .transpose()?;
                (Some(partial), next)
            }
            None => (None, None),
        };

        Ok(Manifest {
            dependency_maps,
            platform,
            extends,
        })
    }
}

pub(super) enum ManifestKey {
    Node,
    Npm,
    Pnpm,
    Yarn,
}

impl fmt::Display for ManifestKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ManifestKey::Node => "node",
            ManifestKey::Npm => "npm",
            ManifestKey::Pnpm => "pnpm",
            ManifestKey::Yarn => "yarn",
        })
    }
}

/// Updates the `volta` hash in the specified manifest with the given key and value
///
/// Will create the `volta` hash if it isn't already present
///
/// If the value is `None`, will remove the key from the hash
pub(super) fn update_manifest(
    file: &Path,
    key: ManifestKey,
    value: Option<&Version>,
) -> Fallible<()> {
    let contents = read_to_string(file).with_context(|| ErrorKind::PackageReadError {
        file: file.to_owned(),
    })?;

    let mut manifest: serde_json::Value =
        serde_json::from_str(&contents).with_context(|| ErrorKind::PackageParseError {
            file: file.to_owned(),
        })?;

    let root = manifest
        .as_object_mut()
        .ok_or_else(|| ErrorKind::PackageParseError {
            file: file.to_owned(),
        })?;

    let key = key.to_string();

    match (value, root.get_mut("volta").and_then(|v| v.as_object_mut())) {
        (Some(v), Some(hash)) => {
            hash.insert(key, Value::String(v.to_string()));
        }
        (None, Some(hash)) => {
            hash.remove(&key);
        }
        (Some(v), None) => {
            let mut map = Map::new();
            map.insert(key, Value::String(v.to_string()));
            root.insert("volta".into(), Value::Object(map));
        }
        (None, None) => {}
    }

    let indent = detect_indent::detect_indent(&contents);
    let mut output = File::create(file).with_context(|| ErrorKind::PackageWriteError {
        file: file.to_owned(),
    })?;
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.indent().as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&output, formatter);
    manifest
        .serialize(&mut ser)
        .with_context(|| ErrorKind::PackageWriteError {
            file: file.to_owned(),
        })?;

    if contents.ends_with('\n') {
        writeln!(output).with_context(|| ErrorKind::PackageWriteError {
            file: file.to_owned(),
        })?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct RawManifest {
    dependencies: Option<HashMap<String, String>>,

    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,

    volta: Option<ToolchainSpec>,
}

impl RawManifest {
    fn from_file(package: &Path) -> Fallible<Self> {
        let file = File::open(package).with_context(|| ErrorKind::PackageReadError {
            file: package.to_owned(),
        })?;

        serde_json::de::from_reader(file).with_context(|| ErrorKind::PackageParseError {
            file: package.to_owned(),
        })
    }
}

#[derive(Default, Deserialize, Serialize)]
struct ToolchainSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pnpm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yarn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extends: Option<PathBuf>,
}

impl ToolchainSpec {
    /// Moves the tool versions into a `PartialPlatform` and returns that along with the `extends` value
    fn parse_split(self) -> Fallible<(PartialPlatform, Option<PathBuf>)> {
        let node = self.node.map(parse_version).transpose()?;
        let npm = self.npm.map(parse_version).transpose()?;
        let pnpm = self.pnpm.map(parse_version).transpose()?;
        let yarn = self.yarn.map(parse_version).transpose()?;

        let platform = PartialPlatform {
            node,
            npm,
            pnpm,
            yarn,
        };

        Ok((platform, self.extends))
    }
}
