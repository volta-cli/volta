// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

use std::process::exit;

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: String,
    flag_global: bool,
}

pub(crate) enum Use {
    Help,
    Global(VersionReq),
    Local(VersionReq),
}

impl Command for Use {
    type Args = Args;

    const USAGE: &'static str = "
Activate a particular toolchain version

Usage:
    notion use [options] <version>
    notion use -h | --help

Options:
    -h, --help     Display this message
    -g, --global   Activate the toolchain globally
";

    fn help() -> Self {
        Use::Help
    }

    fn parse(
        _: Notion,
        Args {
            arg_version,
            flag_global,
        }: Args,
    ) -> Fallible<Self> {
        let requirements = parse_requirements(&arg_version)?;
        Ok(if flag_global {
            Use::Global(requirements)
        } else {
            Use::Local(requirements)
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Use);
        match self {
            Use::Help => {
                Help::Command(CommandName::Use).run(session)?;
            }
            Use::Global(requirements) => {
                session.activate_node(&requirements)?;
            }
            Use::Local(_) => {
                println!("not yet implemented; in the meantime you can modify your package.json.");
                exit(1);
            }
        };
        session.add_event_end(ActivityKind::Use, 0);
        Ok(true)
    }
}
