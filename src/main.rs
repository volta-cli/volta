#[macro_use]
mod command;
mod cli;

use structopt::StructOpt;

use jetson_core::error::{ErrorContext, ErrorReporter};
use jetson_core::session::{ActivityKind, Session};

/// The entry point for the `jetson` CLI.
pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Jetson);

    let jetson = cli::Jetson::from_args();
    let verbose = jetson.verbose;
    let exit_code = jetson.run(&mut session).unwrap_or_else(|err| {
        ErrorReporter::from_flag(env!("CARGO_PKG_VERSION"), verbose)
            .report(ErrorContext::Jetson, &err);
        session.add_event_error(ActivityKind::Jetson, &err);
        err.exit_code()
    });

    session.add_event_end(ActivityKind::Jetson, exit_code);
    session.exit(exit_code);
}
