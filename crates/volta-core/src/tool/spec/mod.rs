use std::fmt::{self, Debug, Display};

use crate::error::ErrorDetails;
use crate::session::Session;
use crate::version::VersionSpec;
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

impl Spec {
    pub fn install(self, session: &mut Session) -> Fallible<()> {
        match self {
            Spec::Node(version) => session.install_node(&version),
            Spec::Npm(_) => Err(ErrorDetails::Unimplemented {
                feature: "Installing npm".into(),
            }
            .into()),
            Spec::Yarn(version) => session.install_yarn(&version),
            Spec::Package(name, version) => session.install_package(name, &version),
        }
    }

    pub fn uninstall(self, session: &mut Session) -> Fallible<()> {
        match self {
            Spec::Node(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling node".into(),
            }
            .into()),
            Spec::Npm(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling npm".into(),
            }
            .into()),
            Spec::Yarn(_) => Err(ErrorDetails::Unimplemented {
                feature: "Uninstalling yarn".into(),
            }
            .into()),
            Spec::Package(name, _) => session.uninstall_package(name),
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
        let s = match self {
            &Spec::Node(ref version) => format!("node version {}", version),
            &Spec::Yarn(ref version) => format!("yarn version {}", version),
            &Spec::Npm(ref version) => format!("npm version {}", version),
            &Spec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}
