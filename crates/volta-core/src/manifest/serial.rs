use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use serde_json::value::Value;

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

    use super::{BinMap, Engines, RawBinManifest};
    use std::collections::HashMap;

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
