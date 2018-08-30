// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_core::catalog::{parse_node_version, parse_yarn_version};
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
            "node" => {
                let node_version = parse_node_version(arg_version)?;
                Ok(Use::Node(parse_requirements(&node_version)?))
            },
            "yarn" => {
                let yarn_version = parse_yarn_version(arg_version)?;
                Ok(Use::Yarn(parse_requirements(&yarn_version)?))
            },
            ref tool => Ok(Use::Other {
                name: tool.to_string(),
                version: parse_requirements(&arg_version)?,
            }),
        }
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
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
        session.add_event_end(ActivityKind::Use, ExitCode::Success);
        Ok(())
    }
}
