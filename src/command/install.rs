use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: String,
}

pub(crate) enum Install {
    Help,
    Default(VersionReq),
}

impl Command for Install {
    type Args = Args;

    const USAGE: &'static str = "
Install a toolchain to the local machine

Usage:
    notion install <version>
    notion install -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Install::Help
    }

    fn parse(_: Notion, Args { arg_version }: Args) -> Fallible<Self> {
        let version = parse_requirements(&arg_version)?;
        Ok(Install::Default(version))
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Install);
        let result = match self {
            Install::Help => Help::Command(CommandName::Install).run(session),
            Install::Default(version) => {
                session.install_node(&version)?;

                Ok(true)
            }
        };
        session.add_event_end(ActivityKind::Install, 0);
        result
    }
}
