use std::fmt::{self, Display};

use super::Tool;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

/// Details required for fetching a 3rd-party Package
#[derive(Debug)]
pub struct PackageDetails {
    pub(crate) version: Version,
    pub(crate) tarball_url: String,
    pub(crate) shasum: String,
}

/// The Tool implementation for fetching and installing 3rd-party packages
#[derive(Debug)]
pub struct Package {
    pub(super) name: String,
    pub(super) details: PackageDetails,
}

impl Package {
    pub fn new(name: String, details: PackageDetails) -> Self {
        Package { name, details }
    }
}

impl Tool for Package {
    fn fetch(self, _session: &mut Session) -> Fallible<()> {
        unimplemented!()
    }
    fn install(self, _session: &mut Session) -> Fallible<()> {
        unimplemented!()
    }
    fn pin(self, _session: &mut Session) -> Fallible<()> {
        unimplemented!()
    }
    fn uninstall(self, _session: &mut Session) -> Fallible<()> {
        unimplemented!()
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version(&self.name, &self.details.version))
    }
}

