pub(crate) mod completions;
pub(crate) mod fetch;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod pin;
pub(crate) mod run;
pub(crate) mod setup;
pub(crate) mod uninstall;
pub(crate) mod r#use;
pub(crate) mod which;

pub(crate) use self::which::Which;
pub(crate) use completions::Completions;
pub(crate) use fetch::Fetch;
pub(crate) use install::Install;
pub(crate) use list::List;
pub(crate) use pin::Pin;
pub(crate) use r#use::Use;
pub(crate) use run::Run;
pub(crate) use setup::Setup;
pub(crate) use uninstall::Uninstall;

use volta_core::error::{ExitCode, Fallible};
use volta_core::session::Session;

/// A Volta command.
pub(crate) trait Command: Sized {
    /// Executes the command. Returns `Ok(true)` if the process should return 0,
    /// `Ok(false)` if the process should return 1, and `Err(e)` if the process
    /// should return `e.exit_code()`.
    fn run(self, session: &mut Session) -> Fallible<ExitCode>;
}
