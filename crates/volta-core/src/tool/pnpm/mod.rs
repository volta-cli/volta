use std::fmt;

use super::{check_fetched, debug_already_fetched, info_fetched, FetchStatus, Tool};
use crate::error::Fallible;
use crate::inventory::pnpm_available;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;

mod fetch;
mod resolve;

pub use resolve::resolve;

pub struct Pnpm {
    pub(super) version: Version,
}

impl Pnpm {
    pub fn new(version: Version) -> Self {
        Pnpm { version }
    }

    pub fn archive_basename(version: &str) -> String {
        format!("pnpm-{}", version)
    }

    pub fn archive_filename(version: &str) -> String {
        format!("{}.tgz", Pnpm::archive_basename(version))
    }

    pub(crate) fn ensure_fetched(&self, session: &mut Session) -> Fallible<()> {
        match check_fetched(|| pnpm_available(&self.version))? {
            FetchStatus::AlreadyFetched => {
                debug_already_fetched(self);
                Ok(())
            }
            FetchStatus::FetchNeeded(_lock) => fetch::fetch(&self.version, session.hooks()?.pnpm()),
        }
    }
}

impl Tool for Pnpm {
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }

    fn install(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!();
    }

    fn pin(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!()
    }
}

impl fmt::Display for Pnpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("pnpm", &self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnpm_archive_basename() {
        assert_eq!(Pnpm::archive_basename("3.4.1"), "pnpm-3.4.1");
    }

    #[test]
    fn test_pnpm_archive_filename() {
        assert_eq!(Pnpm::archive_filename("3.2.4"), "pnpm-3.2.4.tgz");
    }
}
