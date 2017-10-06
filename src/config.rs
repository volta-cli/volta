use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

use toml::Value;

#[cfg(windows)]
use windows;

const PUBLIC_NODE_SERVER_ROOT: &'static str = "https://nodejs.org/dist/";

pub fn public_node_url(version: &str, os: &str, arch: &str) -> String {
    let verbose_root = format!("node-v{}-{}-{}", version, os, arch);
    format!("{}/v{}/{}.tar.gz", PUBLIC_NODE_SERVER_ROOT, version, verbose_root)
}

fn directory_config(dir: &Path) -> Option<PathBuf> {
    let config_path = dir.join(".nodeup.toml");
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

// nodeup_home:
// - unix:    ~/.nodeup
// - windows: %LOCALAPPDATA%\nodeup

// directories:
// - nodeup_bin:      ${nodeup_home}/bin
// - nodeup_binstubs: ${nodeup_home}/opt/bin
// - nodeup_versions: ${nodeup_home}/versions

#[cfg(not(windows))]
fn nodeup_home() -> Option<PathBuf> {
    env::home_dir().and_then(|home| {
        home.join(".nodeup")
    })
}

#[cfg(windows)]
fn nodeup_home() -> Option<PathBuf> {
    Some(Path::new(&windows::get_local_app_data_path())
        .join("nodeup"))
}

fn user_config() -> Option<PathBuf> {
    nodeup_home().map(|nodeup| {
        nodeup.join("config.toml")
    })
}

pub fn nodeup_binstubs() -> Option<PathBuf> {
    nodeup_home().map(|nodeup| {
        nodeup.join("opt")
              .join("bin")
    })
}

pub fn nodeup_versions() -> Option<PathBuf> {
    nodeup_home().map(|nodeup| {
        nodeup.join("versions")
    })
}

pub fn node_install_root() -> Option<PathBuf> {
    nodeup_versions().map(|versions| {
        versions.join("node")
    })
}

pub fn node_version_root(version: &str) -> Option<PathBuf> {
    node_install_root().map(|root| {
        root.join(&format!("v{}", version))
    })
}

pub fn find() -> Option<PathBuf> {
    local_config().or_else(|| user_config())
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
