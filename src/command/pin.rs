use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, display_unknown_error, ErrorContext};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

pub(crate) enum Pin {
    Help,
    Tool(ToolSpec),
}

impl Command for Pin {
    type Args = Args;

    const USAGE: &'static str = "
Select a tool for the current project's toolchain

Usage:
    notion pin <tool> <version>
    notion pin -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Pin::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_tool,
            arg_version,
        }: Args,
    ) -> Fallible<Self> {
        let version = VersionSpec::parse(&arg_version)?;
        Ok(Pin::Tool(ToolSpec::from_str(&arg_tool, version)))
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Pin);
        match self {
            Pin::Help => Help::Command(CommandName::Pin).run(session)?,
            Pin::Tool(toolspec) => session.pin(&toolspec)?,
        };
        if let Some(project) = session.project() {
            let errors = project.autoshim();

            for error in errors {
                if error.is_user_friendly() {
                    display_error(ErrorContext::Notion, &error);
                } else {
                    display_unknown_error(ErrorContext::Notion, &error);
                }
            }
        }
        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(())
    }
}
