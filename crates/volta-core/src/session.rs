//! Provides the `Session` type, which represents the user's state during an
//! execution of a Volta tool, including their current directory, Volta
//! hook configuration, and the state of the local inventory.

use std::fmt::{self, Display, Formatter};
use std::process::exit;
use std::rc::Rc;

use crate::event::EventLog;
use crate::hook::{HookConfig, LazyHookConfig, Publish};
use crate::inventory::{Inventory, LazyInventory};
use crate::platform::PlatformSpec;
use crate::project::{LazyProject, Project};
use crate::toolchain::{LazyToolchain, Toolchain};

use log::debug;
use volta_fail::{ExitCode, Fallible, VoltaError};

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum ActivityKind {
    Fetch,
    Install,
    Uninstall,
    List,
    Current,
    #[cfg(not(feature = "volta-updates"))]
    Deactivate,
    #[cfg(not(feature = "volta-updates"))]
    Activate,
    Default,
    Pin,
    Node,
    Npm,
    Npx,
    Yarn,
    Volta,
    Tool,
    Help,
    Version,
    Binary,
    Shim,
    Completions,
    Which,
    #[cfg(feature = "volta-updates")]
    Setup,
}

impl Display for ActivityKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            ActivityKind::Fetch => "fetch",
            ActivityKind::Install => "install",
            ActivityKind::Uninstall => "uninstall",
            ActivityKind::List => "list",
            ActivityKind::Current => "current",
            #[cfg(not(feature = "volta-updates"))]
            ActivityKind::Deactivate => "deactivate",
            #[cfg(not(feature = "volta-updates"))]
            ActivityKind::Activate => "activate",
            ActivityKind::Default => "default",
            ActivityKind::Pin => "pin",
            ActivityKind::Node => "node",
            ActivityKind::Npm => "npm",
            ActivityKind::Npx => "npx",
            ActivityKind::Yarn => "yarn",
            ActivityKind::Volta => "volta",
            ActivityKind::Tool => "tool",
            ActivityKind::Help => "help",
            ActivityKind::Version => "version",
            ActivityKind::Binary => "binary",
            #[cfg(feature = "volta-updates")]
            ActivityKind::Setup => "setup",
            ActivityKind::Shim => "shim",
            ActivityKind::Completions => "completions",
            ActivityKind::Which => "which",
        };
        f.write_str(s)
    }
}

/// Represents the user's state during an execution of a Volta tool. The session
/// encapsulates a number of aspects of the environment in which the tool was
/// invoked, including:
///
/// - the current directory
/// - the Node project tree that contains the current directory (if any)
/// - the Volta hook configuration
/// - the inventory of locally-fetched Volta tools
pub struct Session {
    hooks: LazyHookConfig,
    inventory: LazyInventory,
    toolchain: LazyToolchain,
    project: LazyProject,
    event_log: EventLog,
}

impl Session {
    /// Constructs a new `Session`.
    pub fn init() -> Session {
        Session {
            hooks: LazyHookConfig::init(),
            inventory: LazyInventory::init(),
            toolchain: LazyToolchain::init(),
            project: LazyProject::init(),
            event_log: EventLog::init(),
        }
    }

    /// Produces a reference to the current Node project, if any.
    pub fn project(&self) -> Fallible<Option<&Project>> {
        self.project.get()
    }

    /// Produces a mutable reference to the current Node project, if any.
    pub fn project_mut(&mut self) -> Fallible<Option<&mut Project>> {
        self.project.get_mut()
    }

    /// Returns the user's currently active platform, if any
    ///
    /// Active platform is determined by first looking at the Project Platform
    ///
    /// - If it exists and has a Yarn version, then we use the project platform
    /// - If it exists but doesn't have a Yarn version, then we merge the two,
    ///   pulling Yarn from the user default platform, if available
    /// - If there is no Project platform, then we use the user Default Platform
    pub fn current_platform(&self) -> Fallible<Option<Rc<PlatformSpec>>> {
        match self.project_platform()? {
            Some(platform) => {
                if platform.yarn.is_none() {
                    if let Some(default) = self.default_platform()? {
                        return Ok(Some(Rc::new(platform.merge(&default))));
                    }
                }
                Ok(Some(Rc::clone(&platform)))
            }
            None => self.default_platform(),
        }
    }

    /// Returns the user's default platform, if any
    pub fn default_platform(&self) -> Fallible<Option<Rc<PlatformSpec>>> {
        let toolchain = self.toolchain.get()?;
        Ok(toolchain.platform())
    }

    /// Returns the current project's pinned platform image, if any.
    pub fn project_platform(&self) -> Fallible<Option<Rc<PlatformSpec>>> {
        if let Some(ref project) = self.project()? {
            return Ok(project.platform());
        }
        Ok(None)
    }

    /// Produces a reference to the current inventory.
    pub fn inventory(&self) -> Fallible<&Inventory> {
        self.inventory.get()
    }

    /// Produces a mutable reference to the current inventory.
    pub fn inventory_mut(&mut self) -> Fallible<&mut Inventory> {
        self.inventory.get_mut()
    }

    /// Produces a reference to the current toolchain (default platform specification)
    pub fn toolchain(&self) -> Fallible<&Toolchain> {
        self.toolchain.get()
    }

    /// Produces a mutable reference to the current toolchain
    pub fn toolchain_mut(&mut self) -> Fallible<&mut Toolchain> {
        self.toolchain.get_mut()
    }

    /// Produces a reference to the hook configuration
    pub fn hooks(&self) -> Fallible<&HookConfig> {
        self.hooks.get()
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        self.event_log.add_event_start(activity_kind)
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: ExitCode) {
        self.event_log.add_event_end(activity_kind, exit_code)
    }
    pub fn add_event_tool_end(&mut self, activity_kind: ActivityKind, exit_code: i32) {
        self.event_log.add_event_tool_end(activity_kind, exit_code)
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &VoltaError) {
        self.event_log.add_event_error(activity_kind, error)
    }

    fn publish_to_event_log(mut self) {
        match publish_plugin(&self.hooks) {
            Ok(plugin) => {
                self.event_log.publish(plugin);
            }
            Err(e) => {
                debug!("Unable to publish event log.\n{}", e);
            }
        }
    }

    pub fn exit(self, code: ExitCode) -> ! {
        self.publish_to_event_log();
        code.exit();
    }

    pub fn exit_tool(self, code: i32) -> ! {
        self.publish_to_event_log();
        exit(code);
    }
}

fn publish_plugin(hooks: &LazyHookConfig) -> Fallible<Option<&Publish>> {
    let hooks = hooks.get()?;
    let publish = hooks.events().and_then(|events| events.publish.as_ref());
    Ok(publish)
}

#[cfg(test)]
pub mod tests {

    use crate::session::Session;
    use std::env;
    use std::path::PathBuf;

    fn fixture_path(fixture_dir: &str) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");
        cargo_manifest_dir.push(fixture_dir);
        cargo_manifest_dir
    }

    #[test]
    fn test_in_pinned_project() {
        let project_pinned = fixture_path("basic");
        env::set_current_dir(&project_pinned).expect("Could not set current directory");
        let pinned_session = Session::init();
        let pinned_platform = pinned_session
            .project_platform()
            .expect("Couldn't create Project");
        assert_eq!(pinned_platform.is_some(), true);

        let project_unpinned = fixture_path("no_toolchain");
        env::set_current_dir(&project_unpinned).expect("Could not set current directory");
        let unpinned_session = Session::init();
        let unpinned_platform = unpinned_session
            .project_platform()
            .expect("Couldn't create Project");
        assert_eq!(unpinned_platform.is_none(), true);
    }
}
