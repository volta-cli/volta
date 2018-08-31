use super::super::manifest;
use version::serial::parse_requirements;

use notion_fail::Fallible;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// wrapper for HashMap to use with deserialization
pub struct BinMap<K, V>(HashMap<K, V>);

impl<K, V> Deref for BinMap<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &HashMap<K, V> {
        &self.0
    }
}

impl<K, V> DerefMut for BinMap<K, V> {
    fn deref_mut(&mut self) -> &mut HashMap<K, V> {
        &mut self.0
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default)]
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,

    pub toolchain: Option<ToolchainManifest>,

    // the "bin" field can be a map or a string
    // (see https://docs.npmjs.com/files/package.json#bin)
    #[serde(default)] // handles Option
    pub bin: Option<BinMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ToolchainManifest {
    pub node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yarn: Option<String>,
}

impl Manifest {
    pub fn into_manifest(self) -> Fallible<manifest::Manifest> {
        let mut map = HashMap::new();
        if let Some(ref bin) = self.bin {
            for (name, path) in bin.iter() {
                // handle case where only the path was given and binary name was unknown
                if name == "" {
                    // npm uses the package name for the binary in this case
                    map.insert(self.name.clone().unwrap(), path.clone());
                } else {
                    map.insert(name.clone(), path.clone());
                }
            }
        }
        Ok(manifest::Manifest {
            toolchain: self.into_toolchain_manifest()?,
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
            bin: map,
        })
    }

    pub fn into_toolchain_manifest(&self) -> Fallible<Option<manifest::ToolchainManifest>> {
        if let Some(toolchain) = &self.toolchain {
            return Ok(Some(manifest::ToolchainManifest {
                node: parse_requirements(&toolchain.node)?,
                node_str: toolchain.node.clone(),
                yarn: if let Some(yarn) = &toolchain.yarn {
                    Some(parse_requirements(&yarn)?)
                } else {
                    None
                },
                yarn_str: toolchain.yarn.clone(),
            }));
        }
        Ok(None)
    }
}

impl ToolchainManifest {
    pub fn new(node_version: String, yarn_version: Option<String>) -> Self {
        ToolchainManifest {
            node: node_version,
            yarn: yarn_version,
        }
    }
}

// (deserialization adapted from https://serde.rs/deserialize-map.html)

struct BinVisitor<K, V> {
    marker: PhantomData<fn() -> BinMap<K, V>>,
}

impl<K, V> BinVisitor<K, V> {
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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
