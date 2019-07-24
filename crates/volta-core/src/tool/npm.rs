use std::fmt::{self, Display};

use super::Tool;
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

impl Display for Npm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("npm", &self.version))
    }
}
