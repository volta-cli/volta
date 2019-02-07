//! Provides types for working with Notion configuration files.

use std::marker::PhantomData;
use std::str::FromStr;

use lazycell::LazyCell;
use toml;

use crate::distro::node::NodeDistro;
use crate::distro::yarn::YarnDistro;
use crate::distro::Distro;
use crate::fs::touch;
use crate::path::user_config_file;
use crate::plugin;
use notion_fail::{Fallible, NotionError, ResultExt};
use readext::ReadExt;

pub(crate) mod serial;

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
    pub node: Option<ToolConfig<NodeDistro>>,
    pub yarn: Option<ToolConfig<YarnDistro>>,
    pub events: Option<EventsConfig>,
}

/// Notion configuration settings relating to the Node executable.
pub struct ToolConfig<D: Distro> {
    /// The plugin for resolving Node versions, if any.
    pub resolve: Option<plugin::ResolvePlugin>,
    /// The plugin for listing the set of Node versions available on the remote server, if any.
    pub ls_remote: Option<plugin::LsRemote>,

    pub phantom: PhantomData<D>,
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
        let serial: serial::Config = toml::from_str(src).unknown()?;
        Ok(serial.into_config()?)
    }
}

/// Notion configuration settings related to events.
pub struct EventsConfig {
    /// The plugin for publishing events, if any.
    pub publish: Option<plugin::Publish>,
}

#[cfg(test)]
pub mod tests {

    use crate::config::Config;
    use crate::plugin;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn test_from_str_urls() {
        let fixture_dir = fixture_path("config");
        let mut urls_file = fixture_dir.clone();

        urls_file.push("urls.toml");
        let node_config: Config = fs::read_to_string(urls_file)
            .expect("Could not read urls.toml")
            .parse()
            .expect("Could not parse urls.toml");
        assert_eq!(
            node_config.node.unwrap().resolve,
            Some(plugin::ResolvePlugin::Url("https://nodejs.org".to_string()))
        );
        assert_eq!(
            node_config.yarn.unwrap().ls_remote,
            Some(plugin::LsRemote::Url("https://yarnpkg.com".to_string()))
        );
        assert_eq!(
            node_config.events.unwrap().publish,
            Some(plugin::Publish::Url("https://google.com".to_string()))
        );
    }

    #[test]
    fn test_from_str_bins() {
        let fixture_dir = fixture_path("config");
        let mut bins_file = fixture_dir.clone();

        bins_file.push("bins.toml");
        let node_config: Config = fs::read_to_string(bins_file)
            .expect("Could not read bins.toml")
            .parse()
            .expect("Could not parse bins.toml");
        assert_eq!(
            node_config.node.unwrap().resolve,
            Some(plugin::ResolvePlugin::Bin("/some/bin/for/node".to_string()))
        );
        assert_eq!(
            node_config.yarn.unwrap().ls_remote,
            Some(plugin::LsRemote::Bin("/bin/to/yarn".to_string()))
        );
        assert_eq!(
            node_config.events.unwrap().publish,
            Some(plugin::Publish::Bin("/events/bin".to_string()))
        );
    }
}
