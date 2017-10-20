use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io;
use std::io::{Read, Write};

use toml::value::{Value, Table};

use version::Version;
use path::user_config_file;
use untoml::{ParseToml, Extract};
use ::ErrorKind::ConfigError as CE;

pub struct State {
    pub node: Option<Version>
}

fn ensure_config_exists(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir)?;
        let mut file = File::create(path)?;
    }
    File::open(path)
}

pub fn state() -> ::Result<State> {
    let path = user_config_file()?;
    let mut file = ensure_config_exists(&path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    parse(&source)
}

fn parse_node_version(root: &mut Table) -> ::Result<Option<Version>> {
    if !root.contains_key("node") {
        return Ok(None);
    }
    let mut node = root.extract("node", CE)?.table("node", CE)?;
    if !node.contains_key("version") {
        return Ok(None);
    }
    let version = node.extract("version", CE)?.string("node.version", CE)?;
    Ok(Some(Version::Public(version)))
}

fn parse(src: &str) -> ::Result<State> {
    let toml = src.parse::<Value>()?;
    let mut root = toml.table("<root>", CE)?;
    let node = parse_node_version(&mut root)?;
    Ok(State { node })
}
