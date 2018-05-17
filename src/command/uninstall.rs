use notion_core::session::{ActivityKind, Session};
use notion_fail::{Fallible, ResultExt};
use semver::Version;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: String,
}

pub(crate) enum Uninstall {
    Help,
    Default(Version),
}

impl Command for Uninstall {
    type Args = Args;

    const USAGE: &'static str = "
Uninstall a toolchain from the local machine

Usage:
    notion uninstall <version>
    notion uninstall -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Uninstall::Help
    }

    fn parse(_: Notion, Args { arg_version }: Args) -> Fallible<Self> {
        let version = Version::parse(&arg_version).unknown()?;
        Ok(Uninstall::Default(version))
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Uninstall);
        let result = match self {
            Uninstall::Help => Help::Command(CommandName::Uninstall).run(session),
            Uninstall::Default(version) => {
                session.catalog_mut()?.uninstall_node(&version)?;
                Ok(true)
            }
        };
        session.add_event_end(ActivityKind::Uninstall, None);
        result
    }
}
