//! Provides the `Session` type, which represents the user's state during an
//! execution of a Notion tool, including their configuration, their current
//! directory, and the state of the local tool catalog.

use catalog::{Catalog, LazyCatalog};
use config::{Config, LazyConfig};
use installer::Installed;
use project::Project;
use std::fmt::{self, Display, Formatter};
use std::process::exit;

use event::EventLog;
use notion_fail::{Fallible, NotionError};
use semver::{Version, VersionReq};

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum ActivityKind {
    Install,
    Uninstall,
    Current,
    Use,
    Node,
    Yarn,
    Notion,
    Tool,
    Help,
    Version,
    Binary,
}

impl Display for ActivityKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &ActivityKind::Install => "install",
            &ActivityKind::Uninstall => "uninstall",
            &ActivityKind::Current => "current",
            &ActivityKind::Use => "use",
            &ActivityKind::Node => "node",
            &ActivityKind::Yarn => "yarn",
            &ActivityKind::Notion => "notion",
            &ActivityKind::Tool => "tool",
            &ActivityKind::Help => "help",
            &ActivityKind::Version => "version",
            &ActivityKind::Binary => "binary",
        };
        f.write_str(s)
    }
}

/// Represents the user's state during an execution of a Notion tool. The session
/// encapsulates a number of aspects of the environment in which the tool was
/// invoked, including:
///     - the current directory
///     - the Node project tree that contains the current directory (if any)
///     - the Notion configuration settings
///     - the catalog of locally-installed Notion tools
pub struct Session {
    config: LazyConfig,
    catalog: LazyCatalog,
    project: Option<Project>,
    event_log: EventLog,
}

impl Session {
    /// Constructs a new `Session`.
    pub fn new() -> Fallible<Session> {
        Ok(Session {
            config: LazyConfig::new(),
            catalog: LazyCatalog::new(),
            project: Project::for_current_dir()?,
            event_log: EventLog::new()?,
        })
    }

    /// Produces a reference to the current Node project, if any.
    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    /// Produces a reference to the current tool catalog.
    pub fn catalog(&self) -> Fallible<&Catalog> {
        self.catalog.get()
    }

    /// Produces a mutable reference to the current tool catalog.
    pub fn catalog_mut(&mut self) -> Fallible<&mut Catalog> {
        self.catalog.get_mut()
    }

    /// Produces a reference to the configuration.
    pub fn config(&self) -> Fallible<&Config> {
        self.config.get()
    }

    /// Produces the version of Node for the current session. If there is an
    /// active project with Notion settings, this will ensure a compatible
    /// version of Node is installed before returning. If there is no active
    /// project with Notion settings, this produces the global version, which
    /// may be `None`.
    pub fn current_node(&mut self) -> Fallible<Option<Version>> {
        if let Some(ref project) = self.project {
            let requirements = &project.manifest().node;
            let catalog = self.catalog.get_mut()?;
            let available = catalog.node.resolve_local(&requirements);

            if available.is_some() {
                return Ok(available);
            }

            let config = self.config.get()?;
            let installed = catalog.install_node(&requirements, config)?;

            return Ok(Some(installed.into_version()));
        }

        Ok(self.catalog()?.node.activated.clone())
    }

    /// Installs a version of Node matching the specified semantic verisoning
    /// requirements.
    pub fn install_node(&mut self, matching: &VersionReq) -> Fallible<Installed> {
        let catalog = self.catalog.get_mut()?;
        let config = self.config.get()?;
        catalog.install_node(matching, config)
    }

    /// Activates a version of Node matching the specified semantic versioning
    /// requirements.
    pub fn activate_node(&mut self, matching: &VersionReq) -> Fallible<()> {
        let catalog = self.catalog.get_mut()?;
        let config = self.config.get()?;
        catalog.activate_node(matching, config)
    }

    /// Produces the version of Yarn for the current session. If there is an
    /// active project with Notion settings, this will ensure a compatible
    /// version of Yarn is installed before returning. If there is no active
    /// project with Notion settings, this produces the global version, which
    /// may be `None`.
    pub fn current_yarn(&mut self) -> Fallible<Option<Version>> {
        if let Some(ref project) = self.project {
            let requirements = &project.manifest().yarn.clone().unwrap();
            let catalog = self.catalog.get_mut()?;
            let available = catalog.yarn.resolve_local(&requirements);

            if available.is_some() {
                return Ok(available);
            }

            let config = self.config.get()?;
            let installed = catalog.install_yarn(&requirements, config)?;

            return Ok(Some(installed.into_version()));
        }

        Ok(self.catalog()?.yarn.activated.clone())
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        self.event_log.add_event_start(activity_kind)
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: i32) {
        self.event_log.add_event_end(activity_kind, exit_code)
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &NotionError) {
        self.event_log.add_event_error(activity_kind, error)
    }

    // send the events from this session to the monitor
    pub fn send_events(&mut self) {
        let command = self.events_command();
        self.event_log.send_events(command)
    }

    // get the .notion.events_plugin string from package.json
    pub fn events_command(&self) -> Option<String> {
        self.project
            .as_ref()
            .and_then(|project| project.manifest().events_plugin.as_ref())
            .map(|plugin| plugin.to_string())
    }

    pub fn exit(mut self, code: i32) -> ! {
        self.send_events();
        exit(code);
    }
}
