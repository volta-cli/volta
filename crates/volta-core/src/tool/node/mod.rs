use std::fmt::{self, Display};

use super::{debug_already_fetched, info_fetched, info_installed, info_pinned, Tool};
use crate::error::ErrorDetails;
use crate::session::Session;
use crate::style::tool_version;
use cfg_if::cfg_if;
use semver::Version;
use volta_fail::Fallible;

mod fetch;
mod resolve;
mod serial;

pub use fetch::load_default_npm_version;
pub use resolve::resolve;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        /// The OS component of a Node distro's filename.
        pub const NODE_DISTRO_OS: &str = "win";
    } else if #[cfg(target_os = "macos")] {
        /// The OS component of a Node distro's filename.
        pub const NODE_DISTRO_OS: &str = "darwin";
    } else if #[cfg(target_os = "linux")] {
        /// The OS component of a Node distro's filename.
        pub const NODE_DISTRO_OS: &str = "linux";
    } else {
        compile_error!("Unsupported operating system (expected Windows, macOS, or Linux).");
    }
}

cfg_if! {
    if #[cfg(target_arch = "x86")] {
        /// The system architecture component of a Node distro's name.
        pub const NODE_DISTRO_ARCH: &str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        /// The system architecture component of a Node distro's name.
        pub const NODE_DISTRO_ARCH: &str = "x64";
    } else if #[cfg(target_arch = "aarch64")] {
        /// The system architecture component of a Node distro's name.
        pub const NODE_DISTRO_ARCH: &str = "arm64";
    } else {
        compile_error!("Unsupported target_arch variant (expected 'x86', 'x64', or 'aarch64').");
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        /// Filename extension for Node distro files.
        pub const NODE_DISTRO_EXTENSION: &str = "zip";
    } else {
        /// Filename extension for Node distro files.
        pub const NODE_DISTRO_EXTENSION: &str = "tar.gz";
    }
}

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

    pub fn archive_basename(version: &str) -> String {
        format!("node-v{}-{}-{}", version, NODE_DISTRO_OS, NODE_DISTRO_ARCH)
    }

    pub fn archive_filename(version: &str) -> String {
        format!(
            "{}.{}",
            Node::archive_basename(version),
            NODE_DISTRO_EXTENSION
        )
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
        if session.project()?.is_some() {
            let node_version = self.fetch_internal(session)?;

            // Note: We know this will succeed, since we checked above
            let project = session.project_mut()?.unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_archive_basename() {
        assert_eq!(
            Node::archive_basename("1.2.3"),
            format!("node-v1.2.3-{}-{}", NODE_DISTRO_OS, NODE_DISTRO_ARCH)
        );
    }

    #[test]
    fn test_node_archive_filename() {
        assert_eq!(
            Node::archive_filename("1.2.3"),
            format!(
                "node-v1.2.3-{}-{}.{}",
                NODE_DISTRO_OS, NODE_DISTRO_ARCH, NODE_DISTRO_EXTENSION
            )
        );
    }
}
