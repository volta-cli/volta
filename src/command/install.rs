use serde::Deserialize;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use result::ResultOptionExt;

use command::{Command, CommandName, Help};
use Notion;

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: Option<String>,
}

pub(crate) enum Install {
    Help,
    Tool(ToolSpec),
}

impl Command for Install {
    type Args = Args;

    const USAGE: &'static str = "
Install a tool in the user toolchain

Usage:
    notion install <tool> [<version>]
    notion install -h | --help

Options:
    -h, --help     Display this message

Supported Tools:
    Currently Notion supports installing `node` and `yarn` - support for more tools is coming soon!
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
        let version = arg_version
            .map(VersionSpec::parse)
            .invert()?
            .unwrap_or_default();

        Ok(Install::Tool(ToolSpec::from_str(&arg_tool, version)))
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Install);
        match self {
            Install::Help => {
                Help::Command(CommandName::Install).run(session)?;
            }
            Install::Tool(toolspec) => {
                session.install(&toolspec)?;
            }
        };
        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(())
    }
}
