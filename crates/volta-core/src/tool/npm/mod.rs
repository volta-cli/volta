use std::fmt::{self, Display};

use super::Tool;
use crate::error::ErrorDetails;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

/// The Tool implementation for fetching and installing Npm (Unimplemented)
#[derive(Debug)]
pub struct Npm {
    pub(super) version: Version,
}

impl Npm {
    pub fn new(version: Version) -> Self {
        Npm { version }
    }
}

impl Tool for Npm {
    // ISSUE(#292) Implement actions for npm
    fn fetch(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        Err(ErrorDetails::Unimplemented {
            feature: "Fetching npm".into(),
        }
        .into())
    }
    fn install(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        Err(ErrorDetails::Unimplemented {
            feature: "Installing npm".into(),
        }
        .into())
    }
    fn pin(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        Err(ErrorDetails::Unimplemented {
            feature: "Pinning npm".into(),
        }
        .into())
    }
}

impl Display for Npm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("npm", &self.version))
    }
}
