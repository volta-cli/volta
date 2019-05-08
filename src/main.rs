#[macro_use]
mod command;
mod cli;

use structopt::StructOpt;

use volta_core::error::{ErrorContext, ErrorReporter};
use volta_core::session::{ActivityKind, Session};

/// The entry point for the `volta` CLI.
pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Volta);

    let volta = cli::Volta::from_args();
    let verbose = volta.verbose;
    let exit_code = volta.run(&mut session).unwrap_or_else(|err| {
        ErrorReporter::from_flag(env!("CARGO_PKG_VERSION"), verbose)
            .report(ErrorContext::Volta, &err);
        session.add_event_error(ActivityKind::Volta, &err);
        err.exit_code()
    });

    session.add_event_end(ActivityKind::Volta, exit_code);
    session.exit(exit_code);
}
