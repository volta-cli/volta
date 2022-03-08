#[macro_use]
mod command;
mod cli;

use std::env;
use structopt::StructOpt;

use volta_core::error::report_error;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_core::session::{ActivityKind, Session};

mod common;
use common::{ensure_layout, Error};

/// The entry point for the `volta` CLI.
pub fn main() {
    let volta = cli::Volta::from_args();
    let verbosity = match (&volta.verbose, &volta.quiet) {
        (false, false) => LogVerbosity::Default,
        (true, false) => LogVerbosity::Verbose,
        (false, true) => LogVerbosity::Quiet,
        (true, true) => unreachable!(
            "StructOpt should prevent the user from providing both --verbose and --quiet"
        ),
    };
    Logger::init(LogContext::Volta, verbosity).expect("Only a single logger should be initialized");

    let mut session = Session::init();
    let volta_argv = env::args().collect::<Vec<String>>().join(" ");
    session.add_event_start(ActivityKind::Volta, volta_argv.clone());

    let result =
        ensure_layout().and_then(|()| volta.run(&mut session, volta_argv).map_err(Error::Volta));
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
