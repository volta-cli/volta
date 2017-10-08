use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io;
use std::io::{Read, Write};

use toml::Value;

#[cfg(not(windows))]
mod unix;

#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

pub fn public_node_url(version: &str, os: &str, arch: &str) -> String {
    let verbose_root = format!("node-v{}-{}-{}", version, os, arch);
    format!("{}/v{}/{}.tar.gz", PUBLIC_NODE_SERVER_ROOT, version, verbose_root)
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

fn open() -> io::Result<File> {
    if let Some(path) = project_config_file() {
        return File::open(path);
    }
    if let Some(path) = user_config_file() {
        return ensure_config_exists(&path);
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "could not determine location of user config"))
}

pub enum Version {
    Public(String)
}

pub struct Config {
    pub node: Version
}

pub fn read() -> Option<Config> {
    let mut cfg_file = match open() {
        Ok(file) => file,
        Err(_) => { return None; }
    };

    let mut cfg_contents = String::new();
    if cfg_file.read_to_string(&mut cfg_contents).is_err() {
        return None;
    }

    let cfg_toml = match cfg_contents.parse::<Value>() {
        Ok(toml) => toml,
        Err(_) => { return None; }
    };

    if let Value::Table(mut root) = cfg_toml {
        if let Some(Value::Table(mut node)) = root.remove("node") {
            if let Some(Value::String(version)) = node.remove("version") {
                return Some(Config {
                    node: Version::Public(version)
                });
            }
        }
    }

    return None;
}
