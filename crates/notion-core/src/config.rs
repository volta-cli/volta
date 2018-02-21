//! Provides types for working with Notion configuration files.

use std::str::FromStr;

use toml;
use lazycell::LazyCell;

use path::user_config_file;
use failure;
use readext::ReadExt;
use serial::touch;
use serial;
use plugin;

/// Lazily loaded Notion configuration settings.
pub struct LazyConfig {
    config: LazyCell<Config>
}

impl LazyConfig {

    /// Constructs a new `LazyConfig` (but does not initialize it).
    pub fn new() -> LazyConfig {
        LazyConfig {
            config: LazyCell::new()
        }
    }

    /// Forces the loading of the configuration settings.
    pub fn get(&self) -> Result<&Config, failure::Error> {
        self.config.try_borrow_with(|| Config::current())
    }
}

/// Notion configuration settings.
pub struct Config {
    pub node: Option<NodeConfig>
}

/// Notion configuration settings relating to the Node executable.
pub struct NodeConfig {
    /// The plugin for resolving Node versions, if any.
    pub resolve: Option<plugin::Resolve>,
    /// The plugin for listing the set of Node versions available on the remote server, if any.
    pub ls_remote: Option<plugin::LsRemote>
}

impl Config {

    /// Returns the current configuration settings, loaded from the filesystem.
    fn current() -> Result<Config, failure::Error> {
        let path = user_config_file()?;
        let src = touch(&path)?.read_into_string()?;
        src.parse()
    }

}

impl FromStr for Config {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::config::Config = toml::from_str(src)?;
        Ok(serial.into_config()?)
    }
}
