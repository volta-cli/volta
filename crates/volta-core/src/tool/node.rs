use std::fmt::{self, Display};

use super::Tool;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

/// The Tool implementation for fetching and installing Node
#[derive(Debug)]
pub struct Node {
    pub(super) version: Version,
}

impl Node {
    pub fn new(version: Version) -> Self {
        Node { version }
    }
}

impl Tool for Node {
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

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("node", &self.version))
    }
}
