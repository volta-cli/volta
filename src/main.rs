#[macro_use]
mod command;
mod cli;

use structopt::StructOpt;

use volta_core::error::report_error;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_core::session::{ActivityKind, Session};

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

    let mut session = Session::new();
    session.add_event_start(ActivityKind::Volta);
    let exit_code = volta.run(&mut session).unwrap_or_else(|err| {
        report_error(env!("CARGO_PKG_VERSION"), &err);
        session.add_event_error(ActivityKind::Volta, &err);
        err.exit_code()
    });

    session.add_event_end(ActivityKind::Volta, exit_code);
    session.exit(exit_code);
}
