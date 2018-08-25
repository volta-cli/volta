use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::catalog::{parse_node_version, parse_yarn_version};
use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible};

use command::{Command, CommandName, Help};
use {CliParseError, Notion};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
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
Fetch a tool to the local machine

Usage:
    notion fetch <tool> <version>
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
            arg_tool,
            arg_version,
        }: Args,
    ) -> Fallible<Self> {
        match &arg_tool[..] {
            "node" => {
                let node_version = parse_node_version(arg_version)?;
                Ok(Fetch::Node(parse_requirements(&node_version)?))
            },
            "yarn" => {
                let yarn_version = parse_yarn_version(arg_version)?;
                Ok(Fetch::Yarn(parse_requirements(&yarn_version)?))
            },
            ref tool => {
                throw!(CliParseError {
                    usage: None,
                    error: format!("no such tool: `{}`", tool),
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
        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        result
    }
}
