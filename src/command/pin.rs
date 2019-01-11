use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, display_unknown_error, ErrorContext};
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible, NotionFail};

use command::{Command, CommandName, Help};
use Notion;

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

// error message for using tools that are not node|yarn
#[derive(Debug, Fail, NotionFail)]
#[fail(
    display = "pinning tool '{}' not yet implemented - for now you can manually edit package.json",
    name
)]
#[notion_fail(code = "NotYetImplemented")]
pub(crate) struct NoCustomPinError {
    pub(crate) name: String,
}

impl NoCustomPinError {
    pub(crate) fn new(name: String) -> Self {
        NoCustomPinError { name: name }
    }
}

pub(crate) enum Pin {
    Help,
    Node(VersionSpec),
    Yarn(VersionSpec),
    Other {
        name: String,
        // not currently used
        #[allow(dead_code)]
        version: VersionSpec,
    },
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
        Ok(match &arg_tool[..] {
            "node" => Pin::Node(VersionSpec::parse(&arg_version)?),
            "yarn" => Pin::Yarn(VersionSpec::parse(&arg_version)?),
            ref tool => Pin::Other {
                name: tool.to_string(),
                version: VersionSpec::parse(&arg_version)?,
            },
        })
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Pin);
        match self {
            Pin::Help => Help::Command(CommandName::Pin).run(session)?,
            Pin::Node(spec) => session.pin_node_version(&spec)?,
            Pin::Yarn(spec) => session.pin_yarn_version(&spec)?,
            Pin::Other { name, .. } => throw!(NoCustomPinError::new(name)),
        };
        if let Some(project) = session.project()? {
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
