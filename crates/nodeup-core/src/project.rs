use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use std::io::{self, Read};
use std::collections::HashMap;

use serde_json;
use serde_json::map::Map;

use version::{Version, VersionSpec};

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn is_node_modules(dir: &Path) -> bool {
   dir.file_name() == Some(OsStr::new("node_modules"))
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, |parent| is_node_modules(parent))
}

pub fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

pub fn find_project_root(mut dir: &Path) -> Option<&Path> {
    while !is_project_root(dir) {
        dir = match dir.parent() {
            Some(parent) => parent,
            None => { return None; }
        }
    }
    return Some(dir);
}

pub struct Manifest {
    pub node: VersionSpec,
    pub yarn: Option<Version>,
    pub dependencies: HashMap<String, String>
}

pub fn find_manifest(dir: &Path) -> ::Result<Option<Manifest>> {
    let root = match find_project_root(dir) {
        Some(root) => root,
        None => { return Ok(None); }
    };

    let file = File::open(dir.join("package.json"))?;

    parse_manifest(serde_json::de::from_reader(file)?)
}

fn parse_manifest(mut value: serde_json::value::Value) -> ::Result<Option<Manifest>> {
    if let serde_json::value::Value::Object(mut props) = value {
        if let Some(nodeup_env) = props.remove("nodeup-env") {
            return parse_nodeup_env(nodeup_env);
        }
    }
    Ok(None)
}

fn parse_nodeup_env(mut env: serde_json::value::Value) -> ::Result<Option<Manifest>> {
    if let serde_json::value::Value::Object(mut props) = env {
        let node = parse_node_version(props.remove("node")
            .ok_or(::ErrorKind::ManifestError(String::from("no node version specified")))?)?;
        // FIXME: parse yarn version
        let dependencies = props.remove("dependencies").map_or(Ok(HashMap::new()), parse_dependencies)?;
        Ok(Some(Manifest { node, yarn: None, dependencies }))
    } else {
        bail!(::ErrorKind::ManifestError(String::from("key 'nodeup-env' is not an object")));
    }
}

fn parse_node_version(version: serde_json::value::Value) -> ::Result<VersionSpec> {
    if let serde_json::value::Value::String(version) = version {
        // FIXME: really parse the version specifier
        Ok(if &version == "latest" {
            VersionSpec::Latest
        } else {
            VersionSpec::Specific(version)
        })
    } else {
        bail!(::ErrorKind::ManifestError(String::from("key 'node' is not a string")));
    }
}

fn parse_dependencies(dependencies: serde_json::value::Value) -> ::Result<HashMap<String, String>> {
    if let serde_json::value::Value::Object(props) = dependencies {
        let mut map = HashMap::new();
        for (key, value) in props.into_iter() {
            if let serde_json::value::Value::String(value) = value {
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
