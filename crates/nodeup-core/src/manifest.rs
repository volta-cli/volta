use std::collections::HashMap;

use serde_json::value::Value;

use version::{Version, VersionSpec};

pub struct Manifest {
    pub node: VersionSpec,
    pub yarn: Option<Version>,
    pub dependencies: HashMap<String, String>
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
