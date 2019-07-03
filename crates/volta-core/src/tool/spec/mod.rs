use std::fmt::{self, Debug, Display};

use crate::error::ErrorDetails;
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
    pub fn resolve(self, _session: &mut Session) -> Fallible<Resolved> {
        // TODO - CPIERCE: Implement Resolvers
        Err(ErrorDetails::Unimplemented {
            feature: "resolve".into(),
        }
        .into())
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
