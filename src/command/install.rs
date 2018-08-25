use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_core::catalog::{parse_node_version, parse_yarn_version};
use notion_fail::{ExitCode, Fallible};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

pub(crate) enum Install {
    Help,
    Node(VersionReq),
    Yarn(VersionReq),
    Other { name: String, version: VersionReq },
}

impl Command for Install {
    type Args = Args;

    const USAGE: &'static str = "
Install a tool in the user toolchain

Usage:
    notion install <tool> <version>
    notion install -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Install::Help
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
                Ok(Install::Node(parse_requirements(&node_version)?))
            },
            "yarn" => {
                let yarn_version = parse_yarn_version(arg_version)?;
                Ok(Install::Yarn(parse_requirements(&yarn_version)?))
            },
            ref tool => Ok(Install::Other {
                name: tool.to_string(),
                version: parse_requirements(&arg_version)?,
            }),
        }
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Install);
        match self {
            Install::Help => {
                Help::Command(CommandName::Install).run(session)?;
            }
            Install::Node(requirements) => {
                session.set_default_node(&requirements)?;
            }
            Install::Yarn(requirements) => {
                session.set_default_yarn(&requirements)?;
            }
            Install::Other {
                name: _,
                version: _,
            } => unimplemented!(),
        };
        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(true)
    }
}
