use std::env;
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io;
use std::io::{Read, Write};

use toml::Value;

use version::Version;
use path::user_config_file;

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
    if let Some(file) = open_local()? {
        Ok(file)
    } else {
        open_global()
    }
}

fn open_local() -> ::Result<Option<File>> {
    if let Some(path) = project_config_file() {
        Ok(Some(File::open(path)?))
    } else {
        Ok(None)
    }
}

fn open_global() -> ::Result<File> {
    let path = user_config_file()?;
    let file = ensure_config_exists(&path)?;
    Ok(file)
}

pub struct Config {
    pub node: Version
}

fn parse(src: &str) -> ::Result<Config> {
    let toml = src.parse::<Value>()?;

    if let Value::Table(mut root) = toml {
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

fn parse_file(mut file: File) -> ::Result<Config> {
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    parse(&source)
}

pub fn read() -> ::Result<Config> {
    parse_file(open()?)
}

pub fn read_local() -> ::Result<Option<Config>> {
    if let Some(file) = open_local()? {
        Ok(Some(parse_file(file)?))
    } else {
        Ok(None)
    }
}

pub fn read_global() -> ::Result<Config> {
    parse_file(open_global()?)
}
