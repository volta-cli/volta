use semver::VersionReq;

use notion_core::serial::version::parse_requirements;
use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: Option<String>,
}

pub(crate) enum Default {
    Help,
    Get,
    Set(VersionReq),
}

impl Command for Default {
    type Args = Args;

    const USAGE: &'static str = "
Get or set the default toolchain

Usage:
    notion default [<version>]
    notion default -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Default::Help
    }

    fn parse(_: Notion, Args { arg_version }: Args) -> Fallible<Self> {
        Ok(if let Some(version) = arg_version {
            let requirements = parse_requirements(&version)?;
            Default::Set(requirements)
        } else {
            Default::Get
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Default);
        match self {
            Default::Help => {
                Help::Command(CommandName::Default).run(session)?;
            }
            Default::Get => match session.catalog()?.node.default {
                Some(ref version) => {
                    println!("v{}", version);
                }
                None => {}
            },
            Default::Set(requirements) => {
                session.set_default_node(&requirements)?;
            }
        };
        session.add_event_end(ActivityKind::Default, 0);
        Ok(true)
    }
}
