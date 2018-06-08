use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

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

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Version);
        let result = match self {
            Version::Help => Help::Command(CommandName::Version).run(session),
            Version::Default => {
                println!("{}", ::VERSION);
                Ok(true)
            }
        };
        session.add_event_end(ActivityKind::Version, 0);
        result
    }
}
