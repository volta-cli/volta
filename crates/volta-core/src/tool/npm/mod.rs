use std::fmt::{self, Display};

use super::{
    debug_already_fetched, info_fetched, info_installed, info_pinned, info_project_version, Tool,
};
use crate::error::ErrorDetails;
use crate::inventory::npm_available;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

mod fetch;
mod resolve;

pub use resolve::resolve;

/// The Tool implementation for fetching and installing npm
#[derive(Debug)]
pub struct Npm {
    pub(super) version: Version,
}

impl Npm {
    pub fn new(version: Version) -> Self {
        Npm { version }
    }

    pub fn archive_basename(version: &str) -> String {
        format!("npm-{}", version)
    }

    pub fn archive_filename(version: &str) -> String {
        format!("{}.tgz", Npm::archive_basename(version))
    }

    pub(crate) fn ensure_fetched(&self, session: &mut Session) -> Fallible<()> {
        if npm_available(&self.version)? {
            debug_already_fetched(self);
            Ok(())
        } else {
            fetch::fetch(&self.version, session.hooks()?.npm())
        }
    }
}

impl Tool for Npm {
    // ISSUE(#292) Implement actions for npm
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        session.toolchain_mut()?.set_active_npm(&self.version)?;

        info_installed(self);

        if let Ok(Some(project)) = session.project_platform() {
            if let Some(npm) = &project.npm {
                info_project_version(tool_version("npm", npm));
            }
        }
        Ok(())
    }
    fn pin(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        if session.project()?.is_some() {
            self.ensure_fetched(session)?;

            // Note: We know this will succeed, since we checked above
            let project = session.project_mut()?.unwrap();
            project.pin_npm(&self.version)?;

            info_pinned(self);
            Ok(())
        } else {
            Err(ErrorDetails::NotInPackage.into())
        }
    }
}

impl Display for Npm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("npm", &self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_archive_basename() {
        assert_eq!(Npm::archive_basename("1.2.3"), "npm-1.2.3");
    }

    #[test]
    fn test_npm_archive_filename() {
        assert_eq!(Npm::archive_filename("1.2.3"), "npm-1.2.3.tgz");
    }
}
