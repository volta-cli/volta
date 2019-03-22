#[macro_use]
mod command;
mod cli;

use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, ErrorContext};

/// The entry point for the `notion` CLI.
pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Notion);

    let notion = cli::Notion::from_args();
    let exit_code = notion.run(&mut session).unwrap_or_else(|err| {
        display_error(ErrorContext::Notion, &err);
        session.add_event_error(ActivityKind::Notion, &err);
        err.exit_code()
    });

    session.add_event_end(ActivityKind::Notion, exit_code);
    session.exit(exit_code);
}
