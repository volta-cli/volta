use std::fmt::{self, Display};

use super::{debug_already_fetched, info_fetched, info_installed, info_pinned, Tool};
use crate::error::ErrorDetails;
use crate::fetch;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

/// The Tool implementation for fetching and installing Yarn
#[derive(Debug)]
pub struct Yarn {
    pub(super) version: Version,
}

impl Yarn {
    pub fn new(version: Version) -> Self {
        Yarn { version }
    }

    fn fetch_internal(&self, session: &mut Session) -> Fallible<()> {
        let inventory = session.inventory()?;
        if inventory.yarn.contains(&self.version) {
            debug_already_fetched(self);
            return Ok(());
        }

        fetch::yarn(&self.version, session.hooks()?.yarn())?;
        session
            .inventory_mut()?
            .yarn
            .versions
            .insert(self.version.clone());

        Ok(())
    }
}

impl Tool for Yarn {
    fn fetch(self, session: &mut Session) -> Fallible<()> {
        self.fetch_internal(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self, session: &mut Session) -> Fallible<()> {
        self.fetch_internal(session)?;

        session.toolchain_mut()?.set_active_yarn(&self.version)?;

        info_installed(self);
        Ok(())
    }
    fn pin(self, session: &mut Session) -> Fallible<()> {
        if let Some(ref project) = session.project()? {
            self.fetch_internal(session)?;
            project.pin_yarn(&self.version)?;

            info_pinned(self);
            Ok(())
        } else {
            Err(ErrorDetails::NotInPackage.into())
        }
    }
    fn uninstall(self, _session: &mut Session) -> Fallible<()> {
        Err(ErrorDetails::Unimplemented {
            feature: "Uninstalling yarn".into(),
        }
        .into())
    }
}

impl Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("yarn", &self.version))
    }
}
