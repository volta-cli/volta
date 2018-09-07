// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use notion_core::session::{ActivityKind, Session};
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible, NotionFail};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

// error message for using tools that are not node|yarn
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "pinning tool '{}' not yet implemented - for now you can manually edit package.json",
       name)]
#[notion_fail(code = "NotYetImplemented")]
pub(crate) struct NoCustomUseError {
    pub(crate) name: String,
}

impl NoCustomUseError {
    pub(crate) fn new(name: String) -> Self {
        NoCustomUseError { name: name }
    }
}

pub(crate) enum Use {
    Help,
    Node(VersionSpec),
    Yarn(VersionSpec),
    Other { name: String, version: VersionSpec },
}

impl Command for Use {
    type Args = Args;

    const USAGE: &'static str = "
Select a tool for the current project's toolchain

Usage:
    notion use <tool> <version>
    notion use -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Use::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_tool,
            arg_version,
        }: Args,
    ) -> Fallible<Self> {
        match &arg_tool[..] {
            "node" => Ok(Use::Node(VersionSpec::parse(&arg_version)?)),
            "yarn" => Ok(Use::Yarn(VersionSpec::parse(&arg_version)?)),
            ref tool => Ok(Use::Other {
                name: tool.to_string(),
                version: VersionSpec::parse(&arg_version)?,
            }),
        }
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Use);
        match self {
            Use::Help => Help::Command(CommandName::Use).run(session)?,
            Use::Node(spec) => session.pin_node_version(&spec)?,
            Use::Yarn(spec) => session.pin_yarn_version(&spec)?,
            Use::Other {
                name: _name,
                version: _,
            } => throw!(NoCustomUseError::new(_name)),
        };
        session.add_event_end(ActivityKind::Use, ExitCode::Success);
        Ok(())
    }
}
