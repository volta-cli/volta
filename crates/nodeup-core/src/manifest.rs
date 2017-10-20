use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use serde_json;
use serde_json::value::Value;

use version::{Version, VersionSpec};
use lockfile::{self, Lockfile};

pub struct Manifest {
    pub node: VersionSpec,
    pub yarn: Option<Version>,
    pub dependencies: HashMap<String, String>
}

// const LATEST_URL: &'static str = "http://nodejs.org/dist/latest/SHASUMS256.txt";

fn resolve_node(spec: &VersionSpec) -> ::Result<lockfile::Entry> {
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
    pub fn resolve(&self) -> ::Result<Lockfile> {
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

pub fn read(project_root: &Path) -> ::Result<Option<Manifest>> {
    let file = File::open(project_root.join("package.json"))?;
    parse(serde_json::de::from_reader(file)?)
}

pub fn parse(value: Value) -> ::Result<Option<Manifest>> {
    if let Value::Object(mut props) = value {
        if let Some(nodeup_env) = props.remove("nodeup-env") {
            return parse_nodeup_env(nodeup_env);
        }
    }
    Ok(None)
}

fn parse_nodeup_env(env: Value) -> ::Result<Option<Manifest>> {
    if let Value::Object(mut props) = env {
        let node = parse_node_version(props.remove("node")
            .ok_or(::ErrorKind::ManifestError(String::from("no node version specified")))?)?;
        // FIXME: parse yarn version
        let dependencies = props.remove("dependencies").map_or(Ok(HashMap::new()), parse_dependencies)?;
        Ok(Some(Manifest { node, yarn: None, dependencies }))
    } else {
        bail!(::ErrorKind::ManifestError(String::from("key 'nodeup-env' is not an object")));
    }
}

fn parse_node_version(version: Value) -> ::Result<VersionSpec> {
    if let Value::String(version) = version {
        Ok(version.parse()?)
    } else {
        bail!(::ErrorKind::ManifestError(String::from("key 'node' is not a string")));
    }
}

fn parse_dependencies(dependencies: Value) -> ::Result<HashMap<String, String>> {
    if let Value::Object(props) = dependencies {
        let mut map = HashMap::new();
        for (key, value) in props.into_iter() {
            if let Value::String(value) = value {
                map.insert(key, value);
            } else {
                bail!(::ErrorKind::ManifestError(format!("dependency value for key '{}' is not a string", key)));
            }
        }
        Ok(map)
    } else {
        bail!(::ErrorKind::ManifestError(String::from("key 'dependencies' is not an object")));
    }
}
