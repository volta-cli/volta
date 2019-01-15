//! Provides types for working with Notion configuration files.

use std::marker::PhantomData;
use std::str::FromStr;

use lazycell::LazyCell;
use toml;

use distro::node::NodeDistro;
use distro::yarn::YarnDistro;
use distro::Distro;
use fs::touch;
use hook;
use notion_fail::{Fallible, NotionError, ResultExt};
use path::user_config_file;
use readext::ReadExt;

pub(crate) mod serial;

/// Lazily loaded Notion configuration settings.
pub struct LazyHookConfig {
    hook_config: LazyCell<HookConfig>,
}

impl LazyHookConfig {
    /// Constructs a new `LazyConfig` (but does not initialize it).
    pub fn new() -> LazyHookConfig {
        LazyHookConfig {
            hook_config: LazyCell::new(),
        }
    }

    /// Forces the loading of the configuration settings.
    pub fn get(&self) -> Fallible<&HookConfig> {
        self.hook_config.try_borrow_with(|| HookConfig::current())
    }
}

/// Notion configuration settings.
pub struct HookConfig {
    pub node: Option<ToolHooks<NodeDistro>>,
    pub yarn: Option<ToolHooks<YarnDistro>>,
    pub events: Option<EventHooks>,
}

/// Notion hooks for an individual tool
pub struct ToolHooks<D: Distro> {
    /// The hook for resolving the URL for a distro version
    pub distro: Option<hook::ToolDistroHook>,
    /// The hook for resolving the URL for the latest version
    pub latest: Option<hook::ToolMetadataHook>,
    /// The hook for resolving the Tool Index URL
    pub index: Option<hook::ToolMetadataHook>,

    pub phantom: PhantomData<D>,
}

impl HookConfig {
    /// Returns the current configuration settings, loaded from the filesystem.
    fn current() -> Fallible<HookConfig> {
        let path = user_config_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
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
    pub publish: Option<hook::Publish>,
}

#[cfg(test)]
pub mod tests {

    use config::HookConfig;
    use hook;
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
            Some(hook::Publish::Url("https://google.com".to_string()))
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
            Some(hook::ToolDistroHook::Bin(
                "/some/bin/for/node/distro".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(hook::ToolMetadataHook::Bin(
                "/some/bin/for/node/latest".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(hook::ToolMetadataHook::Bin(
                "/some/bin/for/node/index".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(hook::ToolDistroHook::Bin("/bin/to/yarn/distro".to_string()))
        );
        assert_eq!(
            yarn.latest,
            Some(hook::ToolMetadataHook::Bin(
                "/bin/to/yarn/latest".to_string()
            ))
        );
        assert_eq!(
            yarn.index,
            Some(hook::ToolMetadataHook::Bin(
                "/bin/to/yarn/index".to_string()
            ))
        );
        assert_eq!(
            hooks.events.unwrap().publish,
            Some(hook::Publish::Bin("/events/bin".to_string()))
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
            Some(hook::ToolDistroHook::Prefix(
                "http://localhost/node/distro/".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(hook::ToolMetadataHook::Prefix(
                "http://localhost/node/latest/".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(hook::ToolMetadataHook::Prefix(
                "http://localhost/node/index/".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(hook::ToolDistroHook::Prefix(
                "http://localhost/yarn/distro/".to_string()
            ))
        );
        assert_eq!(
            yarn.latest,
            Some(hook::ToolMetadataHook::Prefix(
                "http://localhost/yarn/latest/".to_string()
            ))
        );
        assert_eq!(
            yarn.index,
            Some(hook::ToolMetadataHook::Prefix(
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
            Some(hook::ToolDistroHook::Template(
                "http://localhost/node/distro/{version}/".to_string()
            ))
        );
        assert_eq!(
            node.latest,
            Some(hook::ToolMetadataHook::Template(
                "http://localhost/node/latest/{version}/".to_string()
            ))
        );
        assert_eq!(
            node.index,
            Some(hook::ToolMetadataHook::Template(
                "http://localhost/node/index/{version}/".to_string()
            ))
        );
        assert_eq!(
            yarn.distro,
            Some(hook::ToolDistroHook::Template(
                "http://localhost/yarn/distro/{version}/".to_string()
            ))
        );
        assert_eq!(
            yarn.latest,
            Some(hook::ToolMetadataHook::Template(
                "http://localhost/yarn/latest/{version}/".to_string()
            ))
        );
        assert_eq!(
            yarn.index,
            Some(hook::ToolMetadataHook::Template(
                "http://localhost/yarn/index/{version}/".to_string()
            ))
        );
    }
}
