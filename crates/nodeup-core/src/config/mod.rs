use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io;
use std::io::{Read, Write};

use toml::Value;

use version::Version;

#[cfg(not(windows))]
mod unix;

#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

pub fn archive_file(version: &str) -> String {
    format!("{}.{}", archive_root_dir(version), archive_extension())
}

pub fn archive_root_dir(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}

pub fn public_node_url(version: &str, archive: &str) -> String {
    format!("{}v{}/{}", PUBLIC_NODE_SERVER_ROOT, version, archive)
}

fn local_config(dir: &Path) -> Option<PathBuf> {
    let config_path = dir.join(".nodeup.toml");
    if config_path.is_file() {
        Some(config_path)
    } else {
        None
    }
}

fn project_config_file() -> Option<PathBuf> {
    let mut dir_opt = env::current_dir().ok();
    loop {
        match dir_opt {
            Some(dir) => {
                if let config @ Some(_) = local_config(&dir) {
                    return config;
                } else {
                    dir_opt = dir.parent().map(|path| path.to_path_buf());
                }
            }
            None => { return None; }
        }
    }
}

fn ensure_config_exists(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir)?;
        let mut file = File::create(path)?;
        file.write_all(b"[node]\nversion = \"latest\"\n")?;
        file.sync_all()?;
    }
    File::open(path)
}

fn open() -> ::Result<File> {
    if let Some(path) = project_config_file() {
        let file = File::open(path)?;
        return Ok(file);
    }
    let path = user_config_file()?;
    let file = ensure_config_exists(&path)?;
    Ok(file)
}

pub struct Config {
    pub node: Version
}

pub fn read() -> ::Result<Config> {
    let mut cfg_file = open()?;

    let mut cfg_contents = String::new();
    cfg_file.read_to_string(&mut cfg_contents)?;

    let cfg_toml = cfg_contents.parse::<Value>()?;

    if let Value::Table(mut root) = cfg_toml {
        if let Some(Value::Table(mut node)) = root.remove("node") {
            if let Some(Value::String(version)) = node.remove("version") {
                return Ok(Config {
                    node: Version::Public(version)
                });
            } else {
                bail!(::ErrorKind::ConfigError(String::from("node.version")));
            }
        } else {
            bail!(::ErrorKind::ConfigError(String::from("node")));
        }
    } else {
        bail!(::ErrorKind::ConfigError(String::from("<root>")));
    }
}
