use structopt::StructOpt;

use crate::command::Command;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible};

#[macro_export]
macro_rules! usage {
    () => {
        "notion-use

DEPRECATED:
    To install a tool in your user toolchain, use `notion install`.
    To pin a tool in a project toolchain, use `notion pin`.
"
    };
}

#[derive(StructOpt)]
pub(crate) struct Use {}

impl Command for Use {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Help);
        eprintln!(usage!());
        session.add_event_end(ActivityKind::Help, ExitCode::Success);
        Ok(())
    }
}
