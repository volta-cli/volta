use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

use toml::Value;

const CONFIG_FILENAME: &'static str = ".nemo.toml";
const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

pub fn public_node_url(version: &str, os: &str, arch: &str) -> String {
    let verbose_root = format!("node-v{}-{}-{}", version, os, arch);
    format!("{}/v{}/{}.tar.gz", PUBLIC_NODE_SERVER_ROOT, version, verbose_root)
}

fn directory_config(dir: &Path) -> Option<PathBuf> {
    let config_path = dir.join(CONFIG_FILENAME);
    if config_path.is_file() {
        Some(config_path)
    } else {
        None
    }
}

fn local_config() -> Option<PathBuf> {
    let mut dir_opt = env::current_dir().ok();
    loop {
        match dir_opt {
            Some(dir) => {
                if let config @ Some(_) = directory_config(&dir) {
                    return config;
                } else {
                    dir_opt = dir.parent().map(|path| path.to_path_buf());
                }
            }
            None => { return None; }
        }
    }
}

fn home_config() -> Option<PathBuf> {
    env::home_dir().and_then(|home| {
        directory_config(&home)
    })
}

pub fn find() -> Option<PathBuf> {
    local_config().or_else(|| home_config())
}

pub fn nemo_home() -> Option<PathBuf> {
    env::home_dir().map(|home| {
        home.join(".nemo")
    })
}

pub fn nemo_bin() -> Option<PathBuf> {
    nemo_home().map(|home| {
        home.join("bin")
    })
}

pub fn node_install_root() -> Option<PathBuf> {
    nemo_home().map(|nemo| {
        nemo.join("versions").join("node")
    })
}

pub fn node_version_root(version: &str) -> Option<PathBuf> {
    node_install_root().map(|root| {
        root.join(&format!("v{}", version))
    })
}

pub enum Version {
    Public(String)
}

pub struct Config {
    pub node: Version
}

pub fn read() -> Option<Config> {
    let cfg = match find() {
        Some(cfg) => cfg,
        None => { return None; }
    };

    let mut cfg_file = match File::open(cfg) {
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