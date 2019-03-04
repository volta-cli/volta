mod cli;
mod command;

use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, ErrorContext};
use notion_fail::ExitCode;

/// The entry point for the `notion` CLI.
pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Notion);

    let notion = cli::Notion::from_args();
    let exit_code = match notion.command.run(&mut session) {
        Ok(_) => ExitCode::Success,
        Err(err) => {
            display_error(ErrorContext::Notion, &err);
            session.add_event_error(ActivityKind::Notion, &err);
            err.exit_code()
        }
    };

    session.add_event_end(ActivityKind::Notion, exit_code);
    session.exit(exit_code);
}
