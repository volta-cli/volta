//! Provides types for working with Notion hooks.

use std::marker::PhantomData;
use std::str::FromStr;

use lazycell::LazyCell;
use toml;

use crate::distro::node::NodeDistro;
use crate::distro::package::PackageDistro;
use crate::distro::yarn::YarnDistro;
use crate::distro::Distro;
use crate::error::ErrorDetails;
use crate::fs::touch;
use crate::path::user_hooks_file;
use notion_fail::{Fallible, NotionError, ResultExt};
use readext::ReadExt;

pub(crate) mod serial;
pub mod tool;

/// A hook for publishing Notion events.
#[derive(PartialEq, Debug)]
pub enum Publish {
    /// Reports an event by sending a POST request to a URL.
    Url(String),

    /// Reports an event by forking a process and sending the event by IPC.
    Bin(String),
}

/// Lazily loaded Notion hook configuration
pub struct LazyHookConfig {
    settings: LazyCell<HookConfig>,
}

impl LazyHookConfig {
    /// Constructs a new `LazyHookConfig` (but does not initialize it).
    pub fn new() -> LazyHookConfig {
        LazyHookConfig {
            settings: LazyCell::new(),
        }
    }

    /// Forces the loading of the hook configuration
    pub fn get(&self) -> Fallible<&HookConfig> {
        self.settings.try_borrow_with(|| HookConfig::current())
    }
}

/// Notion hook configuration
pub struct HookConfig {
    pub node: Option<ToolHooks<NodeDistro>>,
    pub yarn: Option<ToolHooks<YarnDistro>>,
    pub package: Option<ToolHooks<PackageDistro>>,
    pub events: Option<EventHooks>,
}

/// Notion hooks for an individual tool
pub struct ToolHooks<D: Distro> {
    /// The hook for resolving the URL for a distro version
    pub distro: Option<tool::DistroHook>,
    /// The hook for resolving the URL for the latest version
    pub latest: Option<tool::MetadataHook>,
    /// The hook for resolving the Tool Index URL
    pub index: Option<tool::MetadataHook>,

    pub phantom: PhantomData<D>,
}

impl HookConfig {
    /// Returns the current hooks, loaded from the filesystem.
    fn current() -> Fallible<Self> {
        let path = user_hooks_file()?;
        let src = touch(&path)
            .and_then(|mut file| file.read_into_string())
            .with_context(|_| ErrorDetails::ReadHooksError {
                file: path.to_string_lossy().to_string(),
            })?;
        src.parse()
    }
}

impl FromStr for HookConfig {
    type Err = NotionError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::HookConfig = toml::from_str(src).unknown()?;
        Ok(serial.into_hook_config()?)
    }
}

/// Notion hooks related to events.
pub struct EventHooks {
    /// The hook for publishing events, if any.
    pub publish: Option<Publish>,
}

#[cfg(test)]
pub mod tests {

    use super::{tool, HookConfig, Publish};
    use std::fs;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn test_from_str_event_url() {
        let fixture_dir = fixture_path("hooks");
        let mut url_file = fixture_dir.clone();

        url_file.push("event_url.toml");
        let hooks: HookConfig = fs::read_to_string(url_file)
            .expect("Chould not read event_url.toml")
            .parse()
            .expect("Could not parse event_url.toml");
        assert_eq!(
            hooks.events.unwrap().publish,
            Some(Publish::Url("https://google.com".to_string()))
        );
    }

    #[test]
    fn test_from_str_bins() {
        let fixture_dir = fixture_path("hooks");
        let mut url_file = fixture_dir.clone();

        url_file.push("bins.toml");
        let hooks: HookConfig = fs::read_to_string(url_file)
            .expect("Chould not read bins.toml")
            .parse()
            .expect("Could not parse bins.toml");

        let node = hooks.node.unwrap();
        let yarn = hooks.yarn.unwrap();
        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Bin(
                "/some/bin/for/node/distro".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Bin(
                "/some/bin/for/node/latest".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Bin(
                "/some/bin/for/node/index".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(tool::DistroHook::Bin("/bin/to/yarn/distro".to_string()))
        );
        assert_eq!(
            yarn.latest,
            Some(tool::MetadataHook::Bin("/bin/to/yarn/latest".to_string()))
        );
        assert_eq!(
            yarn.index,
            Some(tool::MetadataHook::Bin("/bin/to/yarn/index".to_string()))
        );
        assert_eq!(
            hooks.events.unwrap().publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }

    #[test]
    fn test_from_str_prefixes() {
        let fixture_dir = fixture_path("hooks");
        let mut url_file = fixture_dir.clone();

        url_file.push("prefixes.toml");
        let hooks: HookConfig = fs::read_to_string(url_file)
            .expect("Chould not read prefixes.toml")
            .parse()
            .expect("Could not parse prefixes.toml");

        let node = hooks.node.unwrap();
        let yarn = hooks.yarn.unwrap();
        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Prefix(
                "http://localhost/node/distro/".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/node/latest/".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/node/index/".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(tool::DistroHook::Prefix(
                "http://localhost/yarn/distro/".to_string()
            ))
        );
        assert_eq!(
            yarn.latest,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/yarn/latest/".to_string()
            ))
        );
        assert_eq!(
            yarn.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/yarn/index/".to_string()
            ))
        );
    }

    #[test]
    fn test_from_str_templates() {
        let fixture_dir = fixture_path("hooks");
        let mut url_file = fixture_dir.clone();

        url_file.push("templates.toml");
        let hooks: HookConfig = fs::read_to_string(url_file)
            .expect("Chould not read templates.toml")
            .parse()
            .expect("Could not parse templates.toml");

        let node = hooks.node.unwrap();
        let yarn = hooks.yarn.unwrap();
        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Template(
                "http://localhost/node/distro/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Template(
                "http://localhost/node/latest/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Template(
                "http://localhost/node/index/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(tool::DistroHook::Template(
                "http://localhost/yarn/distro/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            yarn.latest,
            Some(tool::MetadataHook::Template(
                "http://localhost/yarn/latest/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            yarn.index,
            Some(tool::MetadataHook::Template(
                "http://localhost/yarn/index/{{version}}/".to_string()
            ))
        );
    }
}
