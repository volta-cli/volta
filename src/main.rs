#[macro_use]
mod command;
mod cli;

use clap::Parser;

use volta_core::error::report_error;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_core::session::{ActivityKind, Session};

mod common;
use common::{ensure_layout, Error};

/// The entry point for the `volta` CLI.
pub fn main() {
    let volta = cli::Volta::parse();
    let verbosity = match (&volta.verbose, &volta.quiet) {
        (false, false) => LogVerbosity::Default,
        (true, false) => LogVerbosity::Verbose,
        (false, true) => LogVerbosity::Quiet,
        (true, true) => {
            unreachable!("Clap should prevent the user from providing both --verbose and --quiet")
        }
    };
    Logger::init(LogContext::Volta, verbosity).expect("Only a single logger should be initialized");

    let mut session = Session::init();
    session.add_event_start(ActivityKind::Volta);

    let result = ensure_layout().and_then(|()| volta.run(&mut session).map_err(Error::Volta));
    match result {
        Ok(exit_code) => {
            session.add_event_end(ActivityKind::Volta, exit_code);
            session.exit(exit_code);
        }
        Err(Error::Tool(code)) => {
            session.add_event_tool_end(ActivityKind::Volta, code);
            session.exit_tool(code);
        }
        Err(Error::Volta(err)) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            session.add_event_error(ActivityKind::Volta, &err);
            let code = err.exit_code();
            session.add_event_end(ActivityKind::Volta, code);
            session.exit(code);
        }
    }
}
