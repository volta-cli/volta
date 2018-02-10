use std::str::FromStr;

use toml;
use lazycell::LazyCell;

use path::user_config_file;
use failure;
use readext::ReadExt;
use serial::touch;
use serial;
use plugin;

pub struct LazyConfig {
    config: LazyCell<Config>
}

impl LazyConfig {
    pub fn new() -> LazyConfig {
        LazyConfig {
            config: LazyCell::new()
        }
    }

    pub fn get(&self) -> Result<&Config, failure::Error> {
        self.config.try_borrow_with(|| Config::current())
    }
}

pub struct Config {
    pub node: Option<NodeConfig>
}

pub struct NodeConfig {
    pub resolve: Option<plugin::Resolve>,
    pub ls_remote: Option<plugin::LsRemote>
}

impl Config {
    pub fn current() -> Result<Config, failure::Error> {
        config()
    }
}

// FIXME: delete this once its dependents get deleted, and fold it into Config::current()
pub fn config() -> Result<Config, failure::Error> {
    let path = user_config_file()?;
    let src = touch(&path)?.read_into_string()?;
    src.parse()
}

impl FromStr for Config {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::config::Config = toml::from_str(src)?;
        Ok(serial.into_config()?)
    }
}
