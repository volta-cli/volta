// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_toolchain: String,
    arg_version: String,
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
Select a toolchain for the current project

Usage:
    notion use <toolchain> <version>
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
            arg_toolchain,
            arg_version,
        }: Args,
    ) -> Fallible<Self> {
        match &arg_toolchain[..] {
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
            Use::Help => {
                Help::Command(CommandName::Use).run(session)?;
            }
            Use::Node(_requirements) => unimplemented!(),
            Use::Yarn(_requirements) => unimplemented!(),
            Use::Other {
                name: _,
                version: _,
            } => unimplemented!(),
        };
        session.add_event_end(ActivityKind::Use, 0);
        Ok(true)
    }
}
