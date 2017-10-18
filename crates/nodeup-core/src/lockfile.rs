use std::collections::HashMap;

use toml::value::{Value, Table};

use version::VersionSpec;

pub struct Entry {
    specifier: VersionSpec,
    version: String
}

/*
[node]
specifier = "stable"
version = "8.6.0"

[yarn]
specifier = "1.2"
version = "1.2.1"

[dependencies]
ember = "ember-cli"
*/

pub struct Lockfile {
    pub node: Entry,
    pub yarn: Option<Entry>,
    pub dependencies: HashMap<String, String>
}

trait ParseToml {
    fn table(self, key: &str) -> ::Result<Table>;
    fn string(self, key: &str) -> ::Result<String>;
}

impl ParseToml for Value {
    fn table(self, key: &str) -> ::Result<Table> {
        if let Value::Table(map) = self {
            Ok(map)
        } else {
            bail!(::ErrorKind::LockfileError(String::from(key)));
        }
    }

    fn string(self, key: &str) -> ::Result<String> {
        if let Value::String(string) = self {
            Ok(string)
        } else {
            bail!(::ErrorKind::LockfileError(String::from(key)));
        }
    }
}

trait Extract {
    fn extract(&mut self, key: &str) -> ::Result<Value>;
}

impl Extract for Table {
    fn extract(&mut self, key: &str) -> ::Result<Value> {
        self.remove(key).ok_or(::ErrorKind::LockfileError(String::from(key)).into())
    }
}

pub fn parse(src: &str) -> ::Result<Lockfile> {
    let toml = src.parse::<Value>()?;
    let mut root = toml.table("<root>")?;
    let mut node = root.extract("node")?.table("node")?;
    let version = node.extract("version")?.string("node.version")?;
    let specifier = node.extract("specifier")?.string("node.specifier")?;
    let specifier = specifier.parse::<VersionSpec>()?;
    Ok(Lockfile {
        node: Entry { specifier, version },
        // FIXME: parse these too
        yarn: None,
        dependencies: HashMap::new()
    })
}
