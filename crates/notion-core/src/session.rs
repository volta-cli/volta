//! Provides the `Session` type, which represents the user's state during an
//! execution of a Notion tool, including their configuration, their current
//! directory, and the state of the local tool catalog.

use config::{Config, LazyConfig};
use catalog::{Catalog, LazyCatalog};
use project::Project;
use installer::Installed;

use error::Fallible;
use semver::{Version, VersionReq};

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
    project: Option<Project>
}

impl Session {

    /// Constructs a new `Session`.
    pub fn new() -> Fallible<Session> {
        Ok(Session {
            config: LazyConfig::new(),
            catalog: LazyCatalog::new(),
            project: Project::for_current_dir()?
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
}
