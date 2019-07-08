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

pub enum Resolved {
    Node(Version),
    Npm(Version),
    Yarn(Version),
    Package(String, Version),
}

impl Spec {
    pub fn resolve(self, session: &mut Session) -> Fallible<Resolved> {
        // TODO - CPIERCE: Implement Resolvers
        let version = match self {
            Spec::Node(version) => resolve::node(version, session.hooks()?.node.as_ref()),
            _ => Err(ErrorDetails::Unimplemented {
                feature: "resolve".into(),
            }
            .into()),
        }?;
        Ok(Resolved::Node(version))
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
            Resolved::Node(version)
            | Resolved::Npm(version)
            | Resolved::Yarn(version)
            | Resolved::Package(_, version) => version,
        }
    }
}
