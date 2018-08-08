use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use command::{Command, CommandName, Help};
use {CliParseError, Notion};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_toolchain: String,
    arg_version: String,
}

pub(crate) enum Fetch {
    Help,
    Node(VersionReq),
    Yarn(VersionReq),
}

impl Command for Fetch {
    type Args = Args;

    const USAGE: &'static str = "
Fetch a toolchain to the local machine

Usage:
    notion fetch <toolchain> <version>
    notion fetch -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Fetch::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_toolchain,
            arg_version,
        }: Args,
    ) -> Fallible<Self> {
        match &arg_toolchain[..] {
            "node" => Ok(Fetch::Node(parse_requirements(&arg_version)?)),
            "yarn" => Ok(Fetch::Yarn(parse_requirements(&arg_version)?)),
            ref toolchain => {
                throw!(CliParseError {
                    usage: None,
                    error: format!("no such toolchain: `{}`", toolchain),
                });
            }
        }
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Fetch);
        let result = match self {
            Fetch::Help => Help::Command(CommandName::Fetch).run(session),
            Fetch::Node(version) => {
                session.fetch_node(&version)?;
                Ok(true)
            }
            Fetch::Yarn(version) => {
                session.fetch_yarn(&version)?;
                Ok(true)
            }
        };
        session.add_event_end(ActivityKind::Fetch, 0);
        result
    }
}
