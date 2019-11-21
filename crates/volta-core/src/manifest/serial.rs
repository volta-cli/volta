use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;

use super::super::{manifest, platform};
use crate::version::parse_version;
use log::warn;
use serde;
use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use serde_json::value::Value;
use volta_fail::Fallible;

// wrapper for HashMap to use with deserialization
#[derive(Debug, PartialEq)]
pub struct BinMap<K, V>(HashMap<K, V>)
where
    K: Eq + Hash;

impl<K, V> Deref for BinMap<K, V>
where
    K: Eq + Hash,
{
    type Target = HashMap<K, V>;

    fn deref(&self) -> &HashMap<K, V> {
        &self.0
    }
}

impl<K, V> DerefMut for BinMap<K, V>
where
    K: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut HashMap<K, V> {
        &mut self.0
    }
}

#[derive(serde::Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default)]
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,

    pub toolchain: Option<ToolchainSpec>,

    pub volta: Option<ToolchainSpec>,
}

#[derive(serde::Deserialize)]
pub struct RawBinManifest {
    pub name: Option<String>,

    // the "bin" field can be a map or a string
    // (see https://docs.npmjs.com/files/package.json#bin)
    #[serde(default)] // handles Option
    pub bin: Option<BinMap<String, String>>,

    // We have a custom deserializer here to account for badly-formed `engines`
    // fields in the wild – e.g. if anything besides an object is supplied. See
    // See https://github.com/volta-cli/volta/issues/388 for example.
    #[serde(default, deserialize_with = "Engines::deserialize")]
    pub engines: Option<Engines>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ToolchainSpec {
    pub node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yarn: Option<String>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Engines {
    pub node: String,
}

impl Engines {
    /// Handle deserialization where users may have supplied bad values for the
    /// `node` key in a `package.json`.
    ///
    /// We are intentionally extremely permissive: we simply return `None` for
    /// all scenarios other than finding a valid POJO like `{ node: <spec> }`
    /// because we are happy to use that information if it is available, but if
    /// it is either unavailable or malformed, we simply fall back to our normal
    /// handling.
    pub fn deserialize<'de, D>(d: D) -> Result<Option<Engines>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(d).map(|value: Option<Value>| match value {
            Some(Value::Object(object)) => match object.get("node") {
                Some(Value::String(ref node)) => Some(Engines {
                    node: node.to_string(),
                }),
                _ => None,
            },
            _ => None,
        })
    }
}

impl Manifest {
    pub fn into_manifest(self, package_path: &Path) -> Fallible<manifest::Manifest> {
        Ok(manifest::Manifest {
            platform: self.to_platform(package_path)?.map(Rc::new),
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
        })
    }

    pub fn to_platform(&self, package_path: &Path) -> Fallible<Option<platform::PlatformSpec>> {
        // Backwards compatibility to allow users to upgrade to using the
        // `volta` key simultaneously with the `toolchain` key, but with
        // deprecation warnings about the use of `toolchain`. Prefer the `volta`
        // key if it is set, and provide a deprecation warning if the user is
        // using the `toolchain` key.
        let toolchain = match (&self.volta, &self.toolchain) {
            (Some(volta), None) => Some(volta),
            (Some(volta), Some(_toolchain)) => {
                warn!(
                    "this project (`{}`) is configured with both the deprecated `toolchain` key and the `volta` key; using the versions specified in `volta`.",
                    package_path.display()
                );
                Some(volta)
            }
            (None, Some(toolchain)) => {
                warn!(
                    "this project (`{}`) is configured with the `toolchain` key, which is deprecated and will be removed in a future version. Please switch to `volta` instead.",
                    package_path.display()
                );
                Some(toolchain)
            }
            (None, None) => None,
        };

        if let Some(toolchain) = &toolchain {
            return Ok(Some(platform::PlatformSpec {
                node_runtime: parse_version(&toolchain.node)?,
                npm: if let Some(npm) = &toolchain.npm {
                    Some(parse_version(&npm)?)
                } else {
                    None
                },
                yarn: if let Some(yarn) = &toolchain.yarn {
                    Some(parse_version(&yarn)?)
                } else {
                    None
                },
            }));
        }
        Ok(None)
    }
}

impl From<RawBinManifest> for super::BinManifest {
    fn from(raw: RawBinManifest) -> Self {
        let mut map = HashMap::new();
        if let Some(ref bin) = raw.bin {
            for (name, path) in bin.iter() {
                // handle case where only the path was given and binary name was unknown
                if name == "" {
                    // npm uses the package name for the binary in this case
                    map.insert(raw.name.clone().unwrap(), path.clone());
                } else {
                    map.insert(name.clone(), path.clone());
                }
            }
        }

        super::BinManifest {
            bin: map,
            engine: raw.engines.map(|e| e.node),
        }
    }
}

impl From<Rc<platform::PlatformSpec>> for ToolchainSpec {
    fn from(source: Rc<platform::PlatformSpec>) -> Self {
        ToolchainSpec {
            node: source.node_runtime.to_string(),
            npm: source.npm.as_ref().map(|v| v.to_string()),
            yarn: source.yarn.as_ref().map(|v| v.to_string()),
        }
    }
}

// (deserialization adapted from https://serde.rs/deserialize-map.html)

struct BinVisitor<K, V>
where
    K: Eq + Hash,
{
    marker: PhantomData<fn() -> BinMap<K, V>>,
}

impl<K, V> BinVisitor<K, V>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        BinVisitor {
            marker: PhantomData,
        }
    }
}

// This trait informs Serde how to deserialize BinMap
impl<'de> Deserialize<'de> for BinMap<String, String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bin_visitor: BinVisitor<String, String> = BinVisitor::new();
        deserializer.deserialize_any(bin_visitor)
    }
}

// This trait contains methods to deserialize each type of data
impl<'de, K, V> Visitor<'de> for BinVisitor<K, V>
where
    K: Deserialize<'de> + Hash + Eq,
    V: Deserialize<'de> + Clone,
{
    type Value = BinMap<String, String>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("string or map")
    }

    // handle maps like { "binary-name": "path/to/bin" }
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut bin_map = BinMap(HashMap::new());
        while let Some((name, path)) = access.next_entry()? {
            bin_map.insert(name, path);
        }
        Ok(bin_map)
    }

    // handle strings that are only the path
    fn visit_str<E>(self, bin_path: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut bin_map = BinMap(HashMap::new());
        // There will only be one string here, so add it to the map
        // with "" for the name, since that is unknown here
        bin_map.insert("".to_string(), bin_path.to_string());
        Ok(bin_map)
    }
}

#[cfg(test)]
pub mod tests {

    use super::{BinMap, Engines, Manifest, RawBinManifest};
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_empty_package() {
        let package_empty = "{}";
        // deserializing should not fail
        let _manifest: Manifest =
            serde_json::de::from_str(package_empty).expect("Could not deserialize string");
    }

    #[test]
    fn test_full_package() {
        // all fields populated
        let package_all = r#"{
            "name": "some_package",
            "version": "3.5.2",
            "description": "This is a description",
            "dependencies": { "something": "1.2.3" },
            "devDependencies": { "somethingElse": "1.2.3" },
            "volta": {
                "node": "0.10.5",
                "npm": "1.2.18",
                "yarn": "1.2.1"
            }
        }"#;
        let manifest_all: Manifest =
            serde_json::de::from_str(package_all).expect("Could not deserialize string");

        assert_eq!(manifest_all.name, Some("some_package".to_string()));
        assert_eq!(manifest_all.version, Some("3.5.2".to_string()));
        assert_eq!(
            manifest_all.description,
            Some("This is a description".to_string())
        );
        // (checking the rest of the fields in other tests)
    }

    #[test]
    fn test_package_dependencies() {
        let package_no_deps = r#"{
            "dependencies": {}
        }"#;
        let manifest_no_deps: Manifest =
            serde_json::de::from_str(package_no_deps).expect("Could not deserialize string");
        assert_eq!(manifest_no_deps.dependencies, HashMap::new());

        let package_with_deps = r#"{
            "dependencies": {
                "somedep": "1.3.7"
            }
        }"#;
        let manifest_with_deps: Manifest =
            serde_json::de::from_str(package_with_deps).expect("Could not deserialize string");
        let mut expected_map = HashMap::new();
        expected_map.insert("somedep".to_string(), "1.3.7".to_string());
        assert_eq!(manifest_with_deps.dependencies, expected_map);
    }

    #[test]
    fn test_package_dev_dependencies() {
        let package_no_dev_deps = r#"{
            "devDependencies": {}
        }"#;
        let manifest_no_dev_deps: Manifest =
            serde_json::de::from_str(package_no_dev_deps).expect("Could not deserialize string");
        assert_eq!(manifest_no_dev_deps.dev_dependencies, HashMap::new());

        let package_dev_deps = r#"{
            "devDependencies": {
                "somethingElse": "1.2.3"
            }
        }"#;
        let manifest_dev_deps: Manifest =
            serde_json::de::from_str(package_dev_deps).expect("Could not deserialize string");
        let mut expected_map = HashMap::new();
        expected_map.insert("somethingElse".to_string(), "1.2.3".to_string());
        assert_eq!(manifest_dev_deps.dev_dependencies, expected_map);
    }

    #[test]
    fn test_package_toolchain_with_volta_key() {
        let package_empty_toolchain = r#"{
            "volta": {
            }
        }"#;
        let manifest_empty_toolchain =
            serde_json::de::from_str::<Manifest>(package_empty_toolchain);
        assert!(
            manifest_empty_toolchain.is_err(),
            "Node must be defined under the 'volta' key"
        );

        let package_node_only = r#"{
            "volta": {
                "node": "0.11.4"
            }
        }"#;
        let manifest_node_only: Manifest =
            serde_json::de::from_str(package_node_only).expect("Could not deserialize string");
        assert_eq!(manifest_node_only.volta.unwrap().node, "0.11.4");

        let package_node_npm = r#"{
            "volta": {
                "node": "0.10.5",
                "npm": "1.2.18"
            }
        }"#;
        let manifest_node_npm: Manifest =
            serde_json::de::from_str(package_node_npm).expect("Could not deserialize string");
        let toolchain_node_npm = manifest_node_npm
            .volta
            .expect("Did not parse toolchain correctly");
        assert_eq!(toolchain_node_npm.node, "0.10.5");
        assert_eq!(toolchain_node_npm.npm.unwrap(), "1.2.18");

        let package_yarn_only = r#"{
            "volta": {
                "yarn": "1.2.1"
            }
        }"#;
        let manifest_yarn_only = serde_json::de::from_str::<Manifest>(package_yarn_only);
        assert!(
            manifest_yarn_only.is_err(),
            "Node must be defined under the 'volta' key"
        );

        let package_node_and_yarn = r#"{
            "volta": {
                "node": "0.10.5",
                "npm": "1.2.18",
                "yarn": "1.2.1"
            }
        }"#;
        let manifest_node_and_yarn: Manifest =
            serde_json::de::from_str(package_node_and_yarn).expect("Could not deserialize string");
        let toolchain_node_and_yarn = manifest_node_and_yarn
            .volta
            .expect("Did not parse toolchain correctly");
        assert_eq!(toolchain_node_and_yarn.node, "0.10.5");
        assert_eq!(toolchain_node_and_yarn.yarn.unwrap(), "1.2.1");
    }

    #[test]
    fn test_package_toolchain_with_toolchain_key() {
        let package_empty_toolchain = r#"{
            "toolchain": {
            }
        }"#;
        let manifest_empty_toolchain =
            serde_json::de::from_str::<Manifest>(package_empty_toolchain);
        assert!(
            manifest_empty_toolchain.is_err(),
            "Node must be defined under the 'toolchain' key"
        );

        let package_node_only = r#"{
            "toolchain": {
                "node": "0.11.4"
            }
        }"#;
        let manifest_node_only: Manifest =
            serde_json::de::from_str(package_node_only).expect("Could not deserialize string");
        assert_eq!(manifest_node_only.toolchain.unwrap().node, "0.11.4");

        let package_node_npm = r#"{
            "toolchain": {
                "node": "0.10.5",
                "npm": "1.2.18"
            }
        }"#;
        let manifest_node_npm: Manifest =
            serde_json::de::from_str(package_node_npm).expect("Could not deserialize string");
        let toolchain_node_npm = manifest_node_npm
            .toolchain
            .expect("Did not parse toolchain correctly");
        assert_eq!(toolchain_node_npm.node, "0.10.5");
        assert_eq!(toolchain_node_npm.npm.unwrap(), "1.2.18");

        let package_yarn_only = r#"{
            "toolchain": {
                "yarn": "1.2.1"
            }
        }"#;
        let manifest_yarn_only = serde_json::de::from_str::<Manifest>(package_yarn_only);
        assert!(
            manifest_yarn_only.is_err(),
            "Node must be defined under the 'toolchain' key"
        );

        let package_node_and_yarn = r#"{
            "toolchain": {
                "node": "0.10.5",
                "npm": "1.2.18",
                "yarn": "1.2.1"
            }
        }"#;
        let manifest_node_and_yarn: Manifest =
            serde_json::de::from_str(package_node_and_yarn).expect("Could not deserialize string");
        let toolchain_node_and_yarn = manifest_node_and_yarn
            .toolchain
            .expect("Did not parse toolchain correctly");
        assert_eq!(toolchain_node_and_yarn.node, "0.10.5");
        assert_eq!(toolchain_node_and_yarn.yarn.unwrap(), "1.2.1");
    }

    #[test]
    fn test_package_bin() {
        let package_no_bin = r#"{
            "bin": {
            }
        }"#;
        let manifest_no_bin: RawBinManifest =
            serde_json::de::from_str(package_no_bin).expect("Could not deserialize string");
        assert_eq!(manifest_no_bin.bin.unwrap(), BinMap(HashMap::new()));

        let package_bin_map = r#"{
            "bin": {
                "somebin": "cli.js"
            }
        }"#;
        let manifest_bin_map: RawBinManifest =
            serde_json::de::from_str(package_bin_map).expect("Could not deserialize string");
        let mut expected_bin_map = BinMap(HashMap::new());
        expected_bin_map.insert("somebin".to_string(), "cli.js".to_string());
        assert_eq!(manifest_bin_map.bin.unwrap(), expected_bin_map);

        let package_bin_string = r#"{
            "name": "package_name",
            "bin": "cli.js"
        }"#;
        let manifest_bin_string: RawBinManifest =
            serde_json::de::from_str(package_bin_string).expect("Could not deserialize string");
        let mut expected_bin_string = BinMap(HashMap::new());
        // after serializing the binary name is an empty string for this case
        expected_bin_string.insert("".to_string(), "cli.js".to_string());
        assert_eq!(manifest_bin_string.bin.unwrap(), expected_bin_string);
    }

    #[test]
    fn test_package_engines() {
        let package_with_engines = r#"{
            "engines": {
                "node": "8.* || >= 10.*"
            }
        }"#;

        let manifest_engines: RawBinManifest =
            serde_json::de::from_str(package_with_engines).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines.engines,
            Some(Engines {
                node: "8.* || >= 10.*".to_string()
            })
        );
    }

    #[test]
    fn invalid_engines_fields() {
        let package_engines_string = r#"{
            "engines": "oh, this is weird"
        }"#;
        let manifest_engines_string: RawBinManifest =
            serde_json::de::from_str(package_engines_string).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines_string.engines, None,
            "We intentionally treat strings as `None`."
        );

        let package_engines_array = r#"{
            "engines": ["wat"]
        }"#;
        let manifest_engines_array: RawBinManifest =
            serde_json::de::from_str(package_engines_array).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines_array.engines, None,
            "We intentionally treat arrays as `None`."
        );

        let package_engines_number = r#"{
            "engines": 42
        }"#;
        let manifest_engines_number: RawBinManifest =
            serde_json::de::from_str(package_engines_number).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines_number.engines, None,
            "We intentionally treat numbers as `None`."
        );

        let package_engines_number = r#"{
            "engines": null
        }"#;
        let manifest_engines_number: RawBinManifest =
            serde_json::de::from_str(package_engines_number).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines_number.engines, None,
            "We deserialize `null` as `None` (i.e. normally)."
        );

        let package_engines_number = r#"{
            "engines": false
        }"#;
        let manifest_engines_number: RawBinManifest =
            serde_json::de::from_str(package_engines_number).expect("Could not deserialize string");
        assert_eq!(
            manifest_engines_number.engines, None,
            "We treat booleans as `None`"
        );
    }
}
