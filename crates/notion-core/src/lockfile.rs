use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

use toml::Value;

use version::VersionSpec;
use untoml::{ParseToml, Extract};

use failure;

use super::LockfileError;

fn lockfile_error(msg: String) -> LockfileError {
    LockfileError { msg }
}

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
    pub fn save(&self, project_root: &Path) -> Result<(), failure::Error> {
        let mut file = File::create(project_root.join(".notion.lock"))?;
        file.write_all(b"[node]\n")?;
        file.write_fmt(format_args!("specifier = \"{}\"\n", self.node.specifier))?;
        file.write_fmt(format_args!("version = \"{}\"\n", self.node.version))?;
        // FIXME: serialize the rest
        Ok(())
    }
}

pub fn parse(src: &str) -> Result<Lockfile, failure::Error> {
    let toml = src.parse::<Value>()?;
    let mut root = toml.table("<root>", lockfile_error)?;
    let mut node = root.extract("node", lockfile_error)?.table("node", lockfile_error)?;
    let version = node.extract("version", lockfile_error)?.string("node.version", lockfile_error)?;
    let specifier = node.extract("specifier", lockfile_error)?.string("node.specifier", lockfile_error)?;
    let specifier = specifier.parse::<VersionSpec>()?;
    Ok(Lockfile {
        node: Entry { specifier, version },
        // FIXME: parse these too
        yarn: None,
        dependencies: HashMap::new()
    })
}

pub fn read(project_root: &Path) -> Result<Lockfile, failure::Error> {
    let mut file = File::open(project_root.join(".notion.lock"))?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    parse(&source)
}

pub fn exists(project_root: &Path) -> bool {
    project_root.join(".notion.lock").exists()
}
