//! Provides types for working with Volta hooks.

use std::env;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;

use lazycell::LazyCell;

use crate::distro::node::NodeDistro;
use crate::distro::package::PackageDistro;
use crate::distro::yarn::YarnDistro;
use crate::distro::Distro;
use crate::error::ErrorDetails;
use crate::path::{find_project_dir, user_hooks_file};
use log::debug;
use volta_fail::{Fallible, ResultExt};

pub(crate) mod serial;
pub mod tool;

/// A hook for publishing Volta events.
#[derive(PartialEq, Debug)]
pub enum Publish {
    /// Reports an event by sending a POST request to a URL.
    Url(String),

    /// Reports an event by forking a process and sending the event by IPC.
    Bin(String),
}

/// Lazily loaded Volta hook configuration
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

/// Volta hook configuration
pub struct HookConfig {
    pub node: Option<ToolHooks<NodeDistro>>,
    pub yarn: Option<ToolHooks<YarnDistro>>,
    pub package: Option<ToolHooks<PackageDistro>>,
    pub events: Option<EventHooks>,
}

/// Volta hooks for an individual tool
pub struct ToolHooks<D: Distro> {
    /// The hook for resolving the URL for a distro version
    pub distro: Option<tool::DistroHook>,
    /// The hook for resolving the URL for the latest version
    pub latest: Option<tool::MetadataHook>,
    /// The hook for resolving the Tool Index URL
    pub index: Option<tool::MetadataHook>,

    pub phantom: PhantomData<D>,
}

impl<D: Distro> ToolHooks<D> {
    /// Creates a merged struct, with "right" having precedence over "left".
    fn merge(left: Self, right: Self) -> Self {
        Self {
            distro: right.distro.or(left.distro),
            latest: right.latest.or(left.latest),
            index: right.index.or(left.index),
            phantom: PhantomData,
        }
    }
}

macro_rules! merge_hook_config_field {
    ($left:ident, $right:ident, $field:ident, $type:ident) => {
        match ($left.$field, $right.$field) {
            (Some(left), Some(right)) => Some($type::merge(left, right)),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        }
    };
}

impl HookConfig {
    /// Returns the current hooks, which are a merge between the user hooks and
    /// the project hooks (if any).
    fn current() -> Fallible<Self> {
        let maybe_project_config = Self::for_current_dir()?;
        let maybe_user_config = Self::for_user()?;

        Ok(match (maybe_project_config, maybe_user_config) {
            (Some(project_config), Some(user_config)) => {
                debug!("Merging user and project hooks");
                Self::merge(user_config, project_config)
            }
            (Some(project_config), None) => project_config,
            (None, Some(user_config)) => user_config,
            (None, None) => {
                debug!("No custom hooks found");
                Self {
                    node: None,
                    yarn: None,
                    package: None,
                    events: None,
                }
            }
        })
    }

    /// Returns the per-project hooks for the current directory.
    fn for_current_dir() -> Fallible<Option<Self>> {
        Self::for_dir(&env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?)
    }

    /// Returns the per-project hooks for the specified directory.  If the
    /// specified directory is not itself a project, its ancestors will be
    /// searched.
    fn for_dir(base_dir: &Path) -> Fallible<Option<Self>> {
        match find_project_dir(&base_dir) {
            Some(project_dir) => {
                let path = project_dir.join(".volta").join("hooks.json");
                let hooks_config = Self::from_file(&path)?;

                if hooks_config.is_some() {
                    debug!("Found project hooks in {}", path.display());
                }

                Ok(hooks_config)
            }
            None => Ok(None),
        }
    }

    fn from_file(file_path: &Path) -> Fallible<Option<Self>> {
        if !file_path.is_file() {
            return Ok(None);
        }

        let file = File::open(file_path).with_context(|_| ErrorDetails::ReadHooksError {
            file: file_path.to_path_buf(),
        })?;

        let serial: serial::HookConfig =
            serde_json::de::from_reader(file).with_context(|_| ErrorDetails::ParseHooksError {
                file: file_path.to_path_buf(),
            })?;

        let hooks_path = file_path.parent().unwrap_or(Path::new("/"));

        serial.into_hook_config(hooks_path).map(|hooks| Some(hooks))
    }

    /// Returns the per-user hooks, loaded from the filesystem.
    fn for_user() -> Fallible<Option<Self>> {
        let path = user_hooks_file()?;
        let hooks_config = Self::from_file(&path)?;

        if hooks_config.is_some() {
            debug!("Found user hooks in {}", path.display());
        }

        Ok(hooks_config)
    }

    /// Creates a merged struct, with "right" having precedence over "left".
    fn merge(left: Self, right: Self) -> Self {
        Self {
            node: merge_hook_config_field!(left, right, node, ToolHooks),
            yarn: merge_hook_config_field!(left, right, yarn, ToolHooks),
            package: merge_hook_config_field!(left, right, package, ToolHooks),
            events: merge_hook_config_field!(left, right, events, EventHooks),
        }
    }
}

/// Volta hooks related to events.
pub struct EventHooks {
    /// The hook for publishing events, if any.
    pub publish: Option<Publish>,
}

impl EventHooks {
    /// Creates a merged struct, with "right" having precedence over "left".
    fn merge(left: Self, right: Self) -> Self {
        Self {
            publish: right.publish.or(left.publish),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::{tool, HookConfig, Publish};
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
        let url_file = fixture_dir.join("event_url.json");
        let hooks = HookConfig::from_file(&url_file).unwrap().unwrap();

        assert_eq!(
            hooks.events.unwrap().publish,
            Some(Publish::Url("https://google.com".to_string()))
        );
    }

    #[test]
    fn test_from_str_bins() {
        let fixture_dir = fixture_path("hooks");
        let bin_file = fixture_dir.join("bins.json");
        let hooks = HookConfig::from_file(&bin_file).unwrap().unwrap();
        let node = hooks.node.unwrap();
        let yarn = hooks.yarn.unwrap();

        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Bin {
                bin: "/some/bin/for/node/distro".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/latest".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/index".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            yarn.distro,
            Some(tool::DistroHook::Bin {
                bin: "/bin/to/yarn/distro".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            yarn.latest,
            Some(tool::MetadataHook::Bin {
                bin: "/bin/to/yarn/latest".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            yarn.index,
            Some(tool::MetadataHook::Bin {
                bin: "/bin/to/yarn/index".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            hooks.events.unwrap().publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }

    #[test]
    fn test_from_str_prefixes() {
        let fixture_dir = fixture_path("hooks");
        let prefix_file = fixture_dir.join("prefixes.json");
        let hooks = HookConfig::from_file(&prefix_file).unwrap().unwrap();
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
        let template_file = fixture_dir.join("templates.json");
        let hooks = HookConfig::from_file(&template_file).unwrap().unwrap();
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

    #[test]
    fn test_for_dir() {
        let project_dir = fixture_path("hooks/project");
        let hooks_dir = project_dir.join(".volta");
        let hooks = HookConfig::for_dir(&project_dir)
            .expect("Could not read project hooks.json")
            .expect("Could not find project hooks.json");
        let node = hooks.node.unwrap();

        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Bin {
                bin: "/some/bin/for/node/distro".to_string(),
                base_path: hooks_dir.clone(),
            })
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/latest".to_string(),
                base_path: hooks_dir.clone(),
            })
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/index".to_string(),
                base_path: hooks_dir.clone(),
            })
        );
        assert_eq!(
            hooks.events.unwrap().publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }

    #[test]
    fn test_merge() {
        let fixture_dir = fixture_path("hooks");
        let user_hooks = HookConfig::from_file(&fixture_dir.join("templates.json"))
            .unwrap()
            .unwrap();
        let project_dir = fixture_path("hooks/project");
        let project_hooks_dir = project_dir.join(".volta");
        let project_hooks = HookConfig::for_dir(&project_dir)
            .expect("Could not read project hooks.json")
            .expect("Could not find project hooks.json");
        let merged_hooks = HookConfig::merge(user_hooks, project_hooks);
        let node = merged_hooks.node.expect("No node config found");
        let yarn = merged_hooks.yarn.expect("No yarn config found");

        assert_eq!(
            node.distro,
            Some(tool::DistroHook::Bin {
                bin: "/some/bin/for/node/distro".to_string(),
                base_path: project_hooks_dir.clone(),
            })
        );
        assert_eq!(
            node.latest,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/latest".to_string(),
                base_path: project_hooks_dir.clone(),
            })
        );
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Bin {
                bin: "/some/bin/for/node/index".to_string(),
                base_path: project_hooks_dir.clone(),
            })
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
        assert_eq!(
            merged_hooks.events.expect("No events config found").publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }
}
