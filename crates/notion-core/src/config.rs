//! Provides types for working with Notion configuration files.

use std::marker::PhantomData;
use std::str::FromStr;

use lazycell::LazyCell;
use toml;

use installer::Install;
use installer::node::NodeInstaller;
use installer::yarn::YarnInstaller;
use notion_fail::{Fallible, NotionError, ResultExt};
use path::user_config_file;
use plugin;
use readext::ReadExt;
use serial;
use serial::touch;

/// Lazily loaded Notion configuration settings.
pub struct LazyConfig {
    config: LazyCell<Config>,
}

impl LazyConfig {
    /// Constructs a new `LazyConfig` (but does not initialize it).
    pub fn new() -> LazyConfig {
        LazyConfig {
            config: LazyCell::new(),
        }
    }

    /// Forces the loading of the configuration settings.
    pub fn get(&self) -> Fallible<&Config> {
        self.config.try_borrow_with(|| Config::current())
    }
}

/// Notion configuration settings.
pub struct Config {
    pub node: Option<ToolConfig<NodeInstaller>>,
    pub yarn: Option<ToolConfig<YarnInstaller>>,
}

/// Notion configuration settings relating to the Node executable.
pub struct ToolConfig<I: Install> {
    /// The plugin for resolving Node versions, if any.
    pub resolve: Option<plugin::ResolvePlugin>,
    /// The plugin for listing the set of Node versions available on the remote server, if any.
    pub ls_remote: Option<plugin::LsRemote>,

    pub phantom: PhantomData<I>,
}

impl Config {
    /// Returns the current configuration settings, loaded from the filesystem.
    fn current() -> Fallible<Config> {
        let path = user_config_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
        src.parse()
    }
}

impl FromStr for Config {
    type Err = NotionError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::config::Config = toml::from_str(src).unknown()?;
        Ok(serial.into_config()?)
    }
}
