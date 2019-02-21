mod activate;
mod config;
mod current;
mod deactivate;
mod fetch;
mod install;
mod pin;

pub(crate) use activate::Activate;
pub(crate) use config::Config;
pub(crate) use current::Current;
pub(crate) use deactivate::Deactivate;
pub(crate) use fetch::Fetch;
pub(crate) use install::Install;
pub(crate) use pin::Pin;

use notion_core::session::Session;
use notion_fail::Fallible;

/// A Notion command.
pub(crate) trait Command: Sized {
    /// Executes the command. Returns `Ok(true)` if the process should return 0,
    /// `Ok(false)` if the process should return 1, and `Err(e)` if the process
    /// should return `e.exit_code()`.
    fn run(self, session: &mut Session) -> Fallible<()>;
}
