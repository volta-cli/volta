use std::fmt::{self, Display};

use super::Tool;
use crate::error::{ErrorKind, Fallible};
use crate::session::Session;
use crate::style::tool_version;
use crate::version::VersionSpec;

pub struct Package {
    pub(crate) name: String,
    pub(crate) version: VersionSpec,
}

impl Package {
    pub fn new(name: String, version: VersionSpec) -> Self {
        Package { name, version }
    }
}

impl Tool for Package {
    fn fetch(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!("Implement Fetch using global install");
    }

    fn install(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!("Implement install using global install");
    }

    fn pin(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        Err(ErrorKind::CannotPinPackage { package: self.name }.into())
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.version {
            VersionSpec::None => f.write_str(&self.name),
            _ => f.write_str(&tool_version(&self.name, &self.version)),
        }
    }
}
