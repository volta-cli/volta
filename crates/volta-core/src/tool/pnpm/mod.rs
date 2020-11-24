use std::fmt;

use super::Tool;
use crate::error::Fallible;
use crate::session::Session;
use semver::Version;

mod resolve;

pub use resolve::resolve;

pub struct Pnpm {}

impl Pnpm {
    pub fn new(_version: Version) -> Self {
        println!("Found version: {}", _version.to_string());
        todo!();
    }
}

impl Tool for Pnpm {
    fn fetch(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!()
    }

    fn install(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!()
    }

    fn pin(self: Box<Self>, _session: &mut Session) -> Fallible<()> {
        todo!()
    }
}

impl fmt::Display for Pnpm {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
