use std::fmt::{self, Display};

use super::{debug_already_fetched, info_fetched, info_installed, info_pinned, Tool};
use crate::error::ErrorDetails;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

mod fetch;
mod resolve;
mod serial;

pub use fetch::load_default_npm_version;
pub use resolve::resolve;

/// A full Node version including not just the version of Node itself
/// but also the specific version of npm installed globally with that
/// Node installation.
#[derive(Clone, Debug)]
pub struct NodeVersion {
    /// The version of Node itself.
    pub runtime: Version,
    /// The npm version globally installed with the Node distro.
    pub npm: Version,
}

impl Display for NodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} (with {})",
            tool_version("node", &self.runtime),
            tool_version("npm", &self.npm)
        )
    }
}

/// The Tool implementation for fetching and installing Node
#[derive(Debug)]
pub struct Node {
    pub(super) version: Version,
}

impl Node {
    pub fn new(version: Version) -> Self {
        Node { version }
    }

    pub(crate) fn fetch_internal(&self, session: &mut Session) -> Fallible<NodeVersion> {
        let inventory = session.inventory()?;
        if inventory.node.versions.contains(&self.version) {
            debug_already_fetched(self);
            let npm = fetch::load_default_npm_version(&self.version)?;

            return Ok(NodeVersion {
                runtime: self.version.clone(),
                npm,
            });
        }

        let node_version = fetch::fetch(&self.version, session.hooks()?.node())?;
        session
            .inventory_mut()?
            .node
            .versions
            .insert(self.version.clone());

        Ok(node_version)
    }
}

impl Tool for Node {
    fn fetch(self, session: &mut Session) -> Fallible<()> {
        let node_version = self.fetch_internal(session)?;

        info_fetched(node_version);
        Ok(())
    }
    fn install(self, session: &mut Session) -> Fallible<()> {
        let node_version = self.fetch_internal(session)?;

        session.toolchain_mut()?.set_active_node(&node_version)?;

        info_installed(node_version);
        Ok(())
    }
    fn pin(self, session: &mut Session) -> Fallible<()> {
        if let Some(ref project) = session.project()? {
            let node_version = self.fetch_internal(session)?;

            project.pin_node(&node_version)?;

            info_pinned(node_version);
            Ok(())
        } else {
            Err(ErrorDetails::NotInPackage.into())
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("node", &self.version))
    }
}
