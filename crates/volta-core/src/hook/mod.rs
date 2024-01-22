//! Provides types for working with Volta hooks.

use std::borrow::Cow;
use std::fs::File;
use std::iter::once;
use std::marker::PhantomData;
use std::path::Path;

use crate::error::{Context, ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::project::Project;
use crate::tool::{Node, Npm, Pnpm, Tool};
use log::debug;
use once_cell::unsync::OnceCell;

pub(crate) mod serial;
pub mod tool;

/// A hook for publishing Volta events.
#[derive(PartialEq, Eq, Debug)]
pub enum Publish {
    /// Reports an event by sending a POST request to a URL.
    Url(String),

    /// Reports an event by forking a process and sending the event by IPC.
    Bin(String),
}

/// Lazily loaded Volta hook configuration
pub struct LazyHookConfig {
    settings: OnceCell<HookConfig>,
}

impl LazyHookConfig {
    /// Constructs a new `LazyHookConfig`
    pub fn init() -> LazyHookConfig {
        LazyHookConfig {
            settings: OnceCell::new(),
        }
    }

    /// Forces the loading of the hook configuration from both project-local and user-default hooks
    pub fn get(&self, project: Option<&Project>) -> Fallible<&HookConfig> {
        self.settings
            .get_or_try_init(|| HookConfig::current(project))
    }
}

/// Volta hook configuration
pub struct HookConfig {
    node: Option<ToolHooks<Node>>,
    npm: Option<ToolHooks<Npm>>,
    pnpm: Option<ToolHooks<Pnpm>>,
    yarn: Option<YarnHooks>,
    events: Option<EventHooks>,
}

/// Volta hooks for an individual tool
pub struct ToolHooks<T: Tool> {
    /// The hook for resolving the URL for a distro version
    pub distro: Option<tool::DistroHook>,
    /// The hook for resolving the URL for the latest version
    pub latest: Option<tool::MetadataHook>,
    /// The hook for resolving the Tool Index URL
    pub index: Option<tool::MetadataHook>,

    phantom: PhantomData<T>,
}

/// Volta hooks for Yarn
pub struct YarnHooks {
    /// The hook for resolving the URL for a distro version
    pub distro: Option<tool::DistroHook>,
    /// The hook for resolving the URL for the latest version
    pub latest: Option<tool::MetadataHook>,
    /// The hook for resolving the Tool Index URL
    pub index: Option<tool::YarnIndexHook>,
}

impl<T: Tool> ToolHooks<T> {
    /// Extends this ToolHooks with another, giving precendence to the current instance
    fn merge(self, other: Self) -> Self {
        Self {
            distro: self.distro.or(other.distro),
            latest: self.latest.or(other.latest),
            index: self.index.or(other.index),
            phantom: PhantomData,
        }
    }
}

impl YarnHooks {
    /// Extends this YarnHooks with another, giving precendence to the current instance
    fn merge(self, other: Self) -> Self {
        Self {
            distro: self.distro.or(other.distro),
            latest: self.latest.or(other.latest),
            index: self.index.or(other.index),
        }
    }
}

macro_rules! merge_hooks {
    ($self:ident, $other:ident, $field:ident) => {
        match ($self.$field, $other.$field) {
            (Some(current), Some(other)) => Some(current.merge(other)),
            (Some(single), None) | (None, Some(single)) => Some(single),
            (None, None) => None,
        }
    };
}

impl HookConfig {
    pub fn node(&self) -> Option<&ToolHooks<Node>> {
        self.node.as_ref()
    }

    pub fn npm(&self) -> Option<&ToolHooks<Npm>> {
        self.npm.as_ref()
    }

    pub fn pnpm(&self) -> Option<&ToolHooks<Pnpm>> {
        self.pnpm.as_ref()
    }

    pub fn yarn(&self) -> Option<&YarnHooks> {
        self.yarn.as_ref()
    }

    pub fn events(&self) -> Option<&EventHooks> {
        self.events.as_ref()
    }

    /// Returns the current hooks, which are a merge between the user hooks and
    /// the project hooks (if any).
    fn current(project: Option<&Project>) -> Fallible<Self> {
        let default_hooks_file = volta_home()?.default_hooks_file();

        // Since `from_paths` expects the paths to be sorted in descending precedence order, we
        // include all project hooks first (workspace_roots is already sorted in descending
        // precedence order)
        // See the per-project configuration RFC for more details on the configuration precedence:
        // https://github.com/volta-cli/rfcs/blob/main/text/0033-per-project-config.md#configuration-precedence
        let paths = project
            .into_iter()
            .flat_map(Project::workspace_roots)
            .map(|root| {
                let mut path = root.join(".volta");
                path.push("hooks.json");
                Cow::Owned(path)
            })
            .chain(once(Cow::Borrowed(default_hooks_file)));

        Self::from_paths(paths)
    }

    /// Returns the merged hooks loaded from an iterator of potential hook files
    ///
    /// `paths` should be sorted in order of descending precedence.
    fn from_paths<P, I>(paths: I) -> Fallible<Self>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        paths
            .into_iter()
            .try_fold(None, |acc: Option<Self>, hooks_file| {
                // Try to load the hooks and merge with any already loaded hooks
                match Self::from_file(hooks_file.as_ref())? {
                    Some(hooks) => {
                        debug!(
                            "Loaded custom hooks file: {}",
                            hooks_file.as_ref().display()
                        );
                        Ok(Some(match acc {
                            Some(loaded) => loaded.merge(hooks),
                            None => hooks,
                        }))
                    }
                    None => Ok(acc),
                }
            })
            // If there were no hooks loaded at all, provide a default empty HookConfig
            .map(|maybe_config| {
                maybe_config.unwrap_or_else(|| {
                    debug!("No custom hooks found");
                    Self {
                        node: None,
                        npm: None,
                        pnpm: None,
                        yarn: None,
                        events: None,
                    }
                })
            })
    }

    fn from_file(file_path: &Path) -> Fallible<Option<Self>> {
        if !file_path.is_file() {
            return Ok(None);
        }

        let file = File::open(file_path).with_context(|| ErrorKind::ReadHooksError {
            file: file_path.to_path_buf(),
        })?;

        let raw: serial::RawHookConfig =
            serde_json::de::from_reader(file).with_context(|| ErrorKind::ParseHooksError {
                file: file_path.to_path_buf(),
            })?;

        // Invariant: Since we successfully loaded it, we know we have a valid file path
        let hooks_path = file_path.parent().expect("File paths always have a parent");

        raw.into_hook_config(hooks_path).map(Some)
    }

    /// Merges this HookConfig with another, giving precedence to the current instance
    fn merge(self, other: Self) -> Self {
        Self {
            node: merge_hooks!(self, other, node),
            npm: merge_hooks!(self, other, npm),
            pnpm: merge_hooks!(self, other, pnpm),
            yarn: merge_hooks!(self, other, yarn),
            events: merge_hooks!(self, other, events),
        }
    }
}

/// Format of the registry used for Yarn (Npm or Github)
#[derive(PartialEq, Eq, Debug)]
pub enum RegistryFormat {
    Npm,
    Github,
}

impl RegistryFormat {
    pub fn from_str(raw_format: &str) -> Fallible<RegistryFormat> {
        match raw_format {
            "npm" => Ok(RegistryFormat::Npm),
            "github" => Ok(RegistryFormat::Github),
            other => Err(ErrorKind::InvalidRegistryFormat {
                format: String::from(other),
            }
            .into()),
        }
    }
}

/// Volta hooks related to events.
pub struct EventHooks {
    /// The hook for publishing events, if any.
    pub publish: Option<Publish>,
}

impl EventHooks {
    /// Merges this EventHooks with another, giving precedence to the current instance
    fn merge(self, other: Self) -> Self {
        Self {
            publish: self.publish.or(other.publish),
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::{tool, HookConfig, Publish, RegistryFormat};
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
        let pnpm = hooks.pnpm.unwrap();
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
        // pnpm
        assert_eq!(
            pnpm.distro,
            Some(tool::DistroHook::Bin {
                bin: "/bin/to/pnpm/distro".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            pnpm.latest,
            Some(tool::MetadataHook::Bin {
                bin: "/bin/to/pnpm/latest".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Bin {
                bin: "/bin/to/pnpm/index".to_string(),
                base_path: fixture_dir.clone(),
            })
        );
        // Yarn
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
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Bin {
                    bin: "/bin/to/yarn/index".to_string(),
                    base_path: fixture_dir,
                },
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
        let pnpm = hooks.pnpm.unwrap();
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
        // pnpm
        assert_eq!(
            pnpm.distro,
            Some(tool::DistroHook::Prefix(
                "http://localhost/pnpm/distro/".to_string()
            ))
        );
        assert_eq!(
            pnpm.latest,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/pnpm/latest/".to_string()
            ))
        );
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/pnpm/index/".to_string()
            ))
        );
        // Yarn
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
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Prefix("http://localhost/yarn/index/".to_string())
            })
        );
    }

    #[test]
    fn test_from_str_templates() {
        let fixture_dir = fixture_path("hooks");
        let template_file = fixture_dir.join("templates.json");
        let hooks = HookConfig::from_file(&template_file).unwrap().unwrap();
        let node = hooks.node.unwrap();
        let pnpm = hooks.pnpm.unwrap();
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
        // pnpm
        assert_eq!(
            pnpm.distro,
            Some(tool::DistroHook::Template(
                "http://localhost/pnpm/distro/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.latest,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/latest/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/index/{{version}}/".to_string()
            ))
        );
        // Yarn
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
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Template(
                    "http://localhost/yarn/index/{{version}}/".to_string()
                )
            })
        );
    }

    #[test]
    fn test_from_str_format_npm() {
        let fixture_dir = fixture_path("hooks");
        let format_npm_file = fixture_dir.join("format_npm.json");
        let hooks = HookConfig::from_file(&format_npm_file).unwrap().unwrap();
        let yarn = hooks.yarn.unwrap();
        let node = hooks.node.unwrap();
        let npm = hooks.npm.unwrap();
        let pnpm = hooks.pnpm.unwrap();
        assert_eq!(
            yarn.index,
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Npm,
                metadata: tool::MetadataHook::Prefix("http://localhost/yarn/index/".to_string())
            })
        );
        // node and npm don't have format
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/node/index/".to_string()
            ))
        );
        assert_eq!(
            npm.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/npm/index/".to_string()
            ))
        );
        // pnpm also doesn't have format
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/pnpm/index/".to_string()
            ))
        );
    }

    #[test]
    fn test_from_str_format_github() {
        let fixture_dir = fixture_path("hooks");
        let format_github_file = fixture_dir.join("format_github.json");
        let hooks = HookConfig::from_file(&format_github_file).unwrap().unwrap();
        let yarn = hooks.yarn.unwrap();
        let node = hooks.node.unwrap();
        let npm = hooks.npm.unwrap();
        let pnpm = hooks.pnpm.unwrap();
        assert_eq!(
            yarn.index,
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Prefix("http://localhost/yarn/index/".to_string())
            })
        );
        // node and npm don't have format
        assert_eq!(
            node.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/node/index/".to_string()
            ))
        );
        assert_eq!(
            npm.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/npm/index/".to_string()
            ))
        );
        // pnpm also doesn't have format
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Prefix(
                "http://localhost/pnpm/index/".to_string()
            ))
        );
    }

    #[test]
    fn test_merge() {
        let fixture_dir = fixture_path("hooks");
        let default_hooks = HookConfig::from_file(&fixture_dir.join("templates.json"))
            .unwrap()
            .unwrap();
        let project_hooks_dir = fixture_path("hooks/project/.volta");
        let project_hooks_file = project_hooks_dir.join("hooks.json");
        let project_hooks = HookConfig::from_file(&project_hooks_file)
            .expect("Could not read project hooks.json")
            .expect("Could not find project hooks.json");
        let merged_hooks = project_hooks.merge(default_hooks);
        let node = merged_hooks.node.expect("No node config found");
        let pnpm = merged_hooks.pnpm.expect("No pnpm config found");
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
                base_path: project_hooks_dir,
            })
        );
        // pnpm
        assert_eq!(
            pnpm.distro,
            Some(tool::DistroHook::Template(
                "http://localhost/pnpm/distro/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.latest,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/latest/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/index/{{version}}/".to_string()
            ))
        );
        // Yarn
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
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Template(
                    "http://localhost/yarn/index/{{version}}/".to_string()
                )
            })
        );
        assert_eq!(
            merged_hooks.events.expect("No events config found").publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }

    #[test]
    fn test_from_paths() {
        let project_hooks_dir = fixture_path("hooks/project/.volta");
        let project_hooks_file = project_hooks_dir.join("hooks.json");
        let default_hooks_file = fixture_path("hooks/templates.json");

        let merged_hooks =
            HookConfig::from_paths([project_hooks_file, default_hooks_file]).unwrap();
        let node = merged_hooks.node.expect("No node config found");
        let pnpm = merged_hooks.pnpm.expect("No pnpm config found");
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
                base_path: project_hooks_dir,
            })
        );
        // pnpm
        assert_eq!(
            pnpm.distro,
            Some(tool::DistroHook::Template(
                "http://localhost/pnpm/distro/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.latest,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/latest/{{version}}/".to_string()
            ))
        );
        assert_eq!(
            pnpm.index,
            Some(tool::MetadataHook::Template(
                "http://localhost/pnpm/index/{{version}}/".to_string()
            ))
        );
        // Yarn
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
            Some(tool::YarnIndexHook {
                format: RegistryFormat::Github,
                metadata: tool::MetadataHook::Template(
                    "http://localhost/yarn/index/{{version}}/".to_string()
                )
            })
        );
        assert_eq!(
            merged_hooks.events.expect("No events config found").publish,
            Some(Publish::Bin("/events/bin".to_string()))
        );
    }
}
