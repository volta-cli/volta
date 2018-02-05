use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use serde_json;
use serde_json::value::Value;
use semver::VersionReq;

use version::{Version, VersionSpec};
use lockfile::{self, Lockfile};

use failure;

pub struct Manifest {
    pub node: VersionSpec,
    pub yarn: Option<Version>,
    pub dependencies: HashMap<String, String>
}

// const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

fn resolve_node(spec: &VersionSpec) -> Result<lockfile::Entry, failure::Error> {
    let version = match *spec {
        VersionSpec::Latest => {
            unimplemented!()
        }
        VersionSpec::Path(_) => {
            unimplemented!()
        }
        VersionSpec::Specific(ref version) => {
            version.clone()
        }
    };
    Ok(lockfile::Entry {
        specifier: spec.clone(),
        version: version
    })
}

impl Manifest {
    // FIXME: change to return &VersionReq after we stop using the version crate
    pub fn node_req(&self) -> VersionReq {
        match self.node {
            VersionSpec::Specific(ref version) => {
                VersionReq::parse(version).unwrap()
            }
            _ => { unimplemented!() }
        }
    }

    pub fn resolve(&self) -> Result<Lockfile, failure::Error> {
        Ok(Lockfile {
            node: resolve_node(&self.node)?,
            yarn: None,
            dependencies: HashMap::new()
        })
    }

    pub fn matches(&self, lockfile: &Lockfile) -> bool {
        // FIXME: && compare the others too
        self.node == lockfile.node.specifier
    }
}

pub fn read(project_root: &Path) -> Result<Option<Manifest>, failure::Error> {
    let file = File::open(project_root.join("package.json"))?;
    parse(serde_json::de::from_reader(file)?)
}

pub fn parse(value: Value) -> Result<Option<Manifest>, failure::Error> {
    if let Value::Object(mut props) = value {
        if let Some(notion_config) = props.remove("notion") {
            return parse_notion_config(notion_config);
        }
    }
    Ok(None)
}

fn parse_notion_config(config: Value) -> Result<Option<Manifest>, failure::Error> {
    if let Value::Object(mut props) = config {
        let node = parse_node_version(props.remove("node")
            .ok_or(super::ManifestError {
                msg: String::from("no node version specified")
            })?)?;
        // FIXME: parse yarn version
        let dependencies = props.remove("dependencies").map_or(Ok(HashMap::new()), parse_dependencies)?;
        Ok(Some(Manifest { node, yarn: None, dependencies }))
    } else {
        Err(super::ManifestError {
            msg: String::from("key 'notion' is not an object")
        }.into())
    }
}

fn parse_node_version(version: Value) -> Result<VersionSpec, failure::Error> {
    if let Value::String(version) = version {
        Ok(version.parse()?)
    } else {
        Err(super::ManifestError {
            msg: String::from("key 'node' is not a string")
        }.into())
    }
}

fn parse_dependencies(dependencies: Value) -> Result<HashMap<String, String>, failure::Error> {
    if let Value::Object(props) = dependencies {
        let mut map = HashMap::new();
        for (key, value) in props.into_iter() {
            if let Value::String(value) = value {
                map.insert(key, value);
            } else {
                Err(super::ManifestError {
                    msg: format!("dependency value for key '{}' is not a string", key)
                })?;
            }
        }
        Ok(map)
    } else {
        Err(super::ManifestError {
            msg: String::from("key 'dependencies' is not an object")
        }.into())
    }
}
