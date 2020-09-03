use std::fmt::{self, Display};
use std::path::PathBuf;

use super::Tool;
use crate::error::{ErrorKind, Fallible};
use crate::session::Session;
use crate::style::tool_version;
use crate::version::VersionSpec;

mod install;

/// The Tool implementation for installing 3rd-party global packages
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
        Err(ErrorKind::CannotFetchPackage {
            package: self.to_string(),
        }
        .into())
    }

    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.global_install(session)?;

        todo!("Parse package.json for version / bins and write configs");
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

fn new_package_image_dir(home: &volta_layout::v2::VoltaHome, package_name: &str) -> PathBuf {
    // TODO: An updated layout (and associated migration) will be added in a follow-up PR
    // at which point this function can be removed
    home.package_image_root_dir().join(package_name)
}
