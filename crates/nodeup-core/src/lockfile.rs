use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

use toml::Value;

use version::VersionSpec;
use untoml::{ParseToml, Extract};

use ::ErrorKind::LockfileError as LE;

pub struct Entry {
    pub specifier: VersionSpec,
    pub version: String
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

impl Lockfile {
    pub fn save(&self, project_root: &Path) -> ::Result<()> {
        let mut file = File::create(project_root.join(".nodeup.lock"))?;
        file.write_all(b"[node]\n")?;
        file.write_fmt(format_args!("specifier = \"{}\"\n", self.node.specifier))?;
        file.write_fmt(format_args!("version = \"{}\"\n", self.node.version))?;
        // FIXME: serialize the rest
        Ok(())
    }
}

pub fn parse(src: &str) -> ::Result<Lockfile> {
    let toml = src.parse::<Value>()?;
    let mut root = toml.table("<root>", LE)?;
    let mut node = root.extract("node", LE)?.table("node", LE)?;
    let version = node.extract("version", LE)?.string("node.version", LE)?;
    let specifier = node.extract("specifier", LE)?.string("node.specifier", LE)?;
    let specifier = specifier.parse::<VersionSpec>()?;
    Ok(Lockfile {
        node: Entry { specifier, version },
        // FIXME: parse these too
        yarn: None,
        dependencies: HashMap::new()
    })
}

pub fn read(project_root: &Path) -> ::Result<Lockfile> {
    let mut file = File::open(project_root.join(".nodeup.lock"))?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    parse(&source)
}

pub fn exists(project_root: &Path) -> bool {
    project_root.join(".nodeup.lock").exists()
}
