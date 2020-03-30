use std::fmt::{self, Display};

use super::node::load_default_npm_version;
use super::{
    debug_already_fetched, info_fetched, info_installed, info_pinned, info_project_version, Tool,
};
use crate::error::ErrorDetails;
use crate::inventory::npm_available;
use crate::session::Session;
use crate::style::{success_prefix, tool_version};
use log::info;
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
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        session
            .toolchain_mut()?
            .set_active_npm(Some(self.version.clone()))?;

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
            project.pin_npm(Some(self.version.clone()))?;

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

/// The Tool implementation for setting npm to the version bundled with Node
#[derive(Debug)]
pub struct BundledNpm;

impl Tool for BundledNpm {
    fn fetch(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        info!("Bundled npm is included with Node, use `volta fetch node` to fetch Node");
        Ok(())
    }

    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        let toolchain = session.toolchain_mut()?;

        toolchain.set_active_npm(None)?;

        let bundled_version = toolchain.platform().and_then(|platform| {
            // If we can't load the default Npm version, treat that as no npm being available
            let version = load_default_npm_version(&platform.node).ok()?;
            Some(version.to_string())
        });

        match bundled_version {
            Some(version) => {
                info!(
                    "{} set bundled npm (currently {}) as default",
                    success_prefix(),
                    version
                );
            }
            None => info!("{} set bundled npm as default", success_prefix()),
        }

        Ok(())
    }

    fn pin(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        match session.project_mut()? {
            Some(project) => {
                project.pin_npm(None)?;

                let bundled_version = project.platform().and_then(|platform| {
                    // If we can't load the default Npm version, treat that as no npm being available
                    let version = load_default_npm_version(&platform.node).ok()?;
                    Some(version.to_string())
                });

                match bundled_version {
                    Some(version) => {
                        info!(
                            "{} set package.json to use bundled npm (currently {})",
                            success_prefix(),
                            version
                        );
                    }
                    None => info!("{} set package.json to use bundled npm", success_prefix()),
                }

                Ok(())
            }
            None => Err(ErrorDetails::NotInPackage.into()),
        }
    }
}

impl Display for BundledNpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("npm", "bundled"))
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
