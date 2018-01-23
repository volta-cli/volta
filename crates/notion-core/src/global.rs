use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io;
use std::io::{Read, Write};

use toml::value::{Value, Table};

use version::Version;
use path::user_state_file;
use untoml::{ParseToml, Extract};
use install;
use failure;

use super::StateError;

fn toml_error(key: String) -> StateError {
    StateError {
        msg: format!("invalid state file at key '{}'", key)
    }
}

pub struct State {
    pub node: Option<Version>
}

fn ensure_state_file_exists(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir)?;
        File::create(path)?;
    }
    File::open(path)
}

pub fn state() -> Result<State, failure::Error> {
    let path = user_state_file()?;
    load(&path)
}

fn load(path: &Path) -> Result<State, failure::Error> {
    let mut file = ensure_state_file_exists(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    parse(&source)
}

fn save(path: &Path, state: &State) -> Result<(), failure::Error> {
    let mut file = File::create(path)?;
    if let Some(Version::Public(ref version)) = state.node {
        file.write_all(b"[node]\n")?;
        file.write_fmt(format_args!("version = \"{}\"\n", version))?;
    }
    Ok(())
}

pub fn set(version: Version) -> Result<(), failure::Error> {
    {
        let &Version::Public(ref version) = &version;
        install::by_version(version)?;
    }
    let path = user_state_file()?;
    let mut state = load(&path)?;
    state.node = Some(version);
    save(&path, &state)?;
    Ok(())
}

fn parse_node_version(root: &mut Table) -> Result<Option<Version>, failure::Error> {
    if !root.contains_key("node") {
        return Ok(None);
    }
    let mut node = root.extract("node", toml_error)?.table("node", toml_error)?;
    if !node.contains_key("version") {
        return Ok(None);
    }
    let version = node.extract("version", toml_error)?.string("node.version", toml_error)?;
    Ok(Some(Version::Public(version)))
}

fn parse(src: &str) -> Result<State, failure::Error> {
    let toml = src.parse::<Value>()?;
    let mut root = toml.table("<root>", toml_error)?;
    let node = parse_node_version(&mut root)?;
    Ok(State { node })
}
