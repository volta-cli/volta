use std::fmt::{self, Debug, Display};

use crate::error::ErrorDetails;
use crate::fetch;
use crate::inventory::Inventory;
use crate::resolve;
use crate::session::Session;
use crate::style::tool_version;
use crate::version::VersionSpec;
use log::debug;
use semver::Version;
use volta_fail::Fallible;

pub mod parse;

/// Specification for a tool and its associated version.
#[derive(PartialEq)]
pub enum Spec {
    Node(VersionSpec),
    Npm(VersionSpec),
    Yarn(VersionSpec),
    Package(String, VersionSpec),
}

/// A fully resolved Tool, with all information necessary for fetching
pub enum Resolved {
    Node(Version),
    Npm(Version),
    Yarn(Version),
    Package(String, PackageDetails),
}

#[derive(Debug)]
pub struct PackageDetails {
    pub(crate) version: Version,
    pub(crate) tarball_url: String,
    pub(crate) shasum: String,
}

impl Spec {
    pub fn resolve(self, session: &mut Session) -> Fallible<Resolved> {
        match self {
            Spec::Node(version) => resolve::node(version, session).map(Resolved::Node),
            Spec::Yarn(version) => resolve::yarn(version, session).map(Resolved::Yarn),
            Spec::Package(name, version) => resolve::package(&name, version, session)
                .map(|details| Resolved::Package(name, details)),
            // Note: To preserve error message context, we always resolve Npm to Version 0.0.0
            // This will allow us to show the correct error message based on the user's command
            // e.g. `volta install npm` vs `volta pin npm`
            Spec::Npm(_) => VersionSpec::parse_version("0.0.0").map(Resolved::Npm),
        }
    }
}

impl Debug for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &Spec::Node(ref version) => tool_version("node", version),
            &Spec::Yarn(ref version) => tool_version("yarn", version),
            &Spec::Npm(ref version) => tool_version("npm", version),
            &Spec::Package(ref name, ref version) => tool_version(name, version),
        };
        f.write_str(&s)
    }
}

impl Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl Resolved {
    pub fn fetch(self, session: &mut Session) -> Fallible<()> {
        self.fetch_internal(session)?;
        // Note: Needs to write a message on a successful fetch
        // Note: Needs to update the inventory on a successful fetch
        Err(ErrorDetails::Unimplemented {
            feature: "fetch".into(),
        }
        .into())
    }

    pub fn install(self, _session: &mut Session) -> Fallible<()> {
        // TODO - CPIERCE: Implement extra install logic
        Err(ErrorDetails::Unimplemented {
            feature: "install".into(),
        }
        .into())
    }

    pub fn pin(self, _session: &mut Session) -> Fallible<()> {
        // TODO - CPIERCE: Implement extra pin logic
        Err(ErrorDetails::Unimplemented {
            feature: "pin".into(),
        }
        .into())
    }

    pub fn uninstall(self, _session: &mut Session) -> Fallible<()> {
        // TODO - CPIERCE: Implement uninstallers
        Err(ErrorDetails::Unimplemented {
            feature: "uninstall".into(),
        }
        .into())
    }

    fn fetch_internal(&self, session: &mut Session) -> Fallible<()> {
        let inventory = session.inventory()?;
        if self.check_already_fetched(inventory) {
            debug!("{} has already been fetched, skipping download", self);
            return Ok(());
        }

        match self {
            Resolved::Yarn(ref version) => fetch::yarn(&version, session.hooks()?.yarn()),
            _ => Err(ErrorDetails::Unimplemented {
                feature: "fetch".into(),
            }
            .into()),
        }
    }

    fn check_already_fetched(&self, inventory: &Inventory) -> bool {
        // TODO - CPIERCE: Look into Package existence checks
        match self {
            Resolved::Node(ref version) => inventory.node.contains(version),
            Resolved::Npm(_) => false,
            Resolved::Yarn(ref version) => inventory.yarn.contains(version),
            Resolved::Package(_, _) => false,
        }
    }
}

impl Debug for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &Resolved::Node(ref version) => tool_version("node", version),
            &Resolved::Yarn(ref version) => tool_version("yarn", version),
            &Resolved::Npm(ref version) => tool_version("npm", version),
            &Resolved::Package(ref name, ref details) => tool_version(name, &details.version),
        };
        f.write_str(&s)
    }
}

impl Display for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl From<Resolved> for Version {
    fn from(tool: Resolved) -> Self {
        match tool {
            Resolved::Node(version) | Resolved::Npm(version) | Resolved::Yarn(version) => version,
            Resolved::Package(_, details) => details.version,
        }
    }
}
