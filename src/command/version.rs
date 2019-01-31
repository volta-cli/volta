use serde::Deserialize;

use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible};

use crate::command::{Command, CommandName, Help};
use crate::Notion;

#[derive(Debug, Deserialize)]
pub(crate) struct Args;

pub(crate) enum Version {
    Help,
    Default,
}

impl Command for Version {
    type Args = Args;

    const USAGE: &'static str = "
Display version information

Usage:
    notion version
    notion version -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Version::Help
    }

    fn parse(_: Notion, _: Args) -> Fallible<Version> {
        Ok(Version::Default)
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Version);
        match self {
            Version::Help => Help::Command(CommandName::Version).run(session)?,
            Version::Default => {
                println!("{}", crate::VERSION);
            }
        };
        session.add_event_end(ActivityKind::Version, ExitCode::Success);
        Ok(())
    }
}
