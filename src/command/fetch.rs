use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use command::{Command, CommandName, Help};
use Notion;

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

pub(crate) enum Fetch {
    Help,
    Tool(ToolSpec),
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

        let version = VersionSpec::parse(&arg_version)?;
        Ok(Fetch::Tool(ToolSpec::from_str(&arg_tool, version)))
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Fetch);
        match self {
            Fetch::Help => Help::Command(CommandName::Fetch).run(session)?,
            Fetch::Tool(toolspec) => {
                session.fetch(&toolspec)?;
            }
        };
        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        Ok(())
    }
}
