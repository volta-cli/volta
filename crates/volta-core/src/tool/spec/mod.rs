use std::fmt::{self, Debug, Display};

use crate::error::ErrorDetails;
use crate::resolve;
use crate::session::Session;
use crate::version::VersionSpec;
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
            &Spec::Node(ref version) => format!("node version {}", version),
            &Spec::Yarn(ref version) => format!("yarn version {}", version),
            &Spec::Npm(ref version) => format!("npm version {}", version),
            &Spec::Package(ref name, ref version) => format!("{} version {}", name, version),
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
    pub fn fetch(self, _session: &mut Session) -> Fallible<()> {
        // TODO - CPIERCE: Implement Fetchers
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
}

impl From<Resolved> for Version {
    fn from(tool: Resolved) -> Self {
        match tool {
            Resolved::Node(version) | Resolved::Npm(version) | Resolved::Yarn(version) => version,
            Resolved::Package(_, details) => details.version,
        }
    }
}
