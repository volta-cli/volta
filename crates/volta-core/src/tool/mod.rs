use std::fmt::{self, Display};

use crate::resolve;
use crate::session::Session;
use crate::style::{success_prefix, tool_version};
use crate::version::VersionSpec;
use log::{debug, info};
use semver::Version;
use volta_fail::Fallible;

mod node;
mod npm;
mod package;
mod parse;
mod yarn;

pub use node::{Node, NodeVersion};
pub use npm::Npm;
pub use package::{Package, PackageDetails};
pub use yarn::Yarn;

#[inline]
fn debug_already_fetched<T: Display + Sized>(tool: T) {
    debug!("{} has already been fetched, skipping download", tool);
}

#[inline]
fn info_installed<T: Display + Sized>(tool: T) {
    info!("{} installed and set {} as default", success_prefix(), tool);
}

#[inline]
fn info_fetched<T: Display + Sized>(tool: T) {
    info!("{} fetched {} to the local system", success_prefix(), tool);
}

#[inline]
fn info_pinned<T: Display + Sized>(tool: T) {
    info!("{} pinned {} in package.json", success_prefix(), tool);
}

/// Trait representing all of the actions that can be taken with a tool
pub trait Tool: Display {
    /// Fetch a Tool into the local inventory
    fn fetch(self, session: &mut Session) -> Fallible<()>;
    /// Install a tool, making it the default so it is available everywhere on the user's machine
    fn install(self, session: &mut Session) -> Fallible<()>;
    /// Pin a tool in the local project so that it is usable within the project
    fn pin(self, session: &mut Session) -> Fallible<()>;
    /// Uninstall a tool, removing it from the local inventory
    fn uninstall(self, session: &mut Session) -> Fallible<()>;
}

/// Specification for a tool and its associated version.
#[derive(Debug, PartialEq)]
pub enum Spec {
    Node(VersionSpec),
    Npm(VersionSpec),
    Yarn(VersionSpec),
    Package(String, VersionSpec),
}

/// A fully resolved Tool, with all information necessary for fetching
#[derive(Debug)]
pub enum Resolved {
    Node(Node),
    Npm(Npm),
    Yarn(Yarn),
    Package(Package),
}

impl Spec {
    /// Resolve a tool spec into a fully realized Tool that can be fetched
    pub fn resolve(self, session: &mut Session) -> Fallible<Resolved> {
        match self {
            Spec::Node(version) => {
                resolve::node(version, session).map(|version| Resolved::Node(Node::new(version)))
            }
            Spec::Yarn(version) => {
                resolve::yarn(version, session).map(|version| Resolved::Yarn(Yarn::new(version)))
            }
            Spec::Package(name, version) => resolve::package(&name, version, session)
                .map(|details| Resolved::Package(Package::new(name, details))),
            // ISSUE (#292): To preserve error message context, we always resolve Npm to Version 0.0.0
            // This will allow us to show the correct error message based on the user's command
            // e.g. `volta install npm` vs `volta pin npm`
            Spec::Npm(_) => {
                VersionSpec::parse_version("0.0.0").map(|version| Resolved::Npm(Npm::new(version)))
            }
        }
    }
}

impl Resolved {
    /// Fetch a Tool into the local inventory
    pub fn fetch(self, session: &mut Session) -> Fallible<()> {
        match self {
            Resolved::Node(node) => node.fetch(session),
            Resolved::Npm(npm) => npm.fetch(session),
            Resolved::Yarn(yarn) => yarn.fetch(session),
            Resolved::Package(package) => package.fetch(session),
        }
    }

    /// Install a tool, making it the default so it is available everywhere on the user's machine
    pub fn install(self, session: &mut Session) -> Fallible<()> {
        match self {
            Resolved::Node(node) => node.install(session),
            Resolved::Npm(npm) => npm.install(session),
            Resolved::Yarn(yarn) => yarn.install(session),
            Resolved::Package(package) => package.install(session),
        }
    }

    /// Pin a tool in the local project so that it is usable within the project
    pub fn pin(self, session: &mut Session) -> Fallible<()> {
        match self {
            Resolved::Node(node) => node.pin(session),
            Resolved::Npm(npm) => npm.pin(session),
            Resolved::Yarn(yarn) => yarn.pin(session),
            Resolved::Package(package) => package.pin(session),
        }
    }

    /// Uninstall a tool, removing it from the local inventory
    pub fn uninstall(self, session: &mut Session) -> Fallible<()> {
        match self {
            Resolved::Node(node) => node.uninstall(session),
            Resolved::Npm(npm) => npm.uninstall(session),
            Resolved::Yarn(yarn) => yarn.uninstall(session),
            Resolved::Package(package) => package.uninstall(session),
        }
    }
}

impl Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Spec::Node(ref version) => tool_version("node", version),
            Spec::Npm(ref version) => tool_version("npm", version),
            Spec::Yarn(ref version) => tool_version("yarn", version),
            Spec::Package(ref name, ref version) => tool_version(name, version),
        };
        f.write_str(&s)
    }
}

impl From<Resolved> for Version {
    fn from(tool: Resolved) -> Self {
        match tool {
            Resolved::Node(Node { version })
            | Resolved::Npm(Npm { version })
            | Resolved::Yarn(Yarn { version }) => version,
            Resolved::Package(Package { details, .. }) => details.version,
        }
    }
}
