// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible, NotionFail};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_tool: String,
    arg_version: String,
}

// error message for using tools that are not node|yarn
#[derive(Fail, Debug)]
#[fail(display = "pinning tool '{}' not yet implemented - for now you can manually edit package.json",
       name)]
pub(crate) struct NoCustomUseError {
    pub(crate) name: String,
}

impl NoCustomUseError {
    pub(crate) fn new(name: String) -> Self {
        NoCustomUseError { name: name }
    }
}

impl_notion_fail!(NoCustomUseError, ExitCode::NotYetImplemented);

pub(crate) enum Use {
    Help,
    Node(VersionReq),
    Yarn(VersionReq),
    Other { name: String, version: VersionReq },
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
            "node" => Ok(Use::Node(parse_requirements(&arg_version)?)),
            "yarn" => Ok(Use::Yarn(parse_requirements(&arg_version)?)),
            ref tool => Ok(Use::Other {
                name: tool.to_string(),
                version: parse_requirements(&arg_version)?,
            }),
        }
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Use);
        match self {
            Use::Help => Help::Command(CommandName::Use).run(session)?,
            Use::Node(requirements) => session.pin_node_version(&requirements)?,
            Use::Yarn(requirements) => session.pin_yarn_version(&requirements)?,
            Use::Other {
                name: _name,
                version: _,
            } => throw!(NoCustomUseError::new(_name)),
        };
        session.add_event_end(ActivityKind::Use, 0);
        Ok(true)
    }
}
