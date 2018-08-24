use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible};

use command::{Command, CommandName, Config, Current, Deactivate, Fetch, Install, Shim, Use,
              Version};
use {CliParseError, Notion};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_command: Option<String>,
}

pub(crate) enum Help {
    Notion,
    Command(CommandName),
}

impl Command for Help {
    type Args = Args;

    const USAGE: &'static str = "
Get some help with a notion command

Usage:
    notion help [<command>]
    notion help -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Help::Command(CommandName::Help)
    }

    fn parse(_: Notion, Args { arg_command }: Args) -> Fallible<Help> {
        Ok(match arg_command {
            None => Help::Notion,
            Some(command) => {
                if let Ok(name) = command.parse() {
                    Help::Command(name)
                } else {
                    throw!(CliParseError {
                        usage: None,
                        error: format!("no such command: `{}`", command),
                    });
                }
            }
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Help);
        eprintln!(
            "{}",
            match self {
                Help::Notion => Notion::USAGE,
                Help::Command(CommandName::Use) => Use::USAGE,
                Help::Command(CommandName::Config) => Config::USAGE,
                Help::Command(CommandName::Current) => Current::USAGE,
                Help::Command(CommandName::Deactivate) => Deactivate::USAGE,
                Help::Command(CommandName::Help) => Help::USAGE,
                Help::Command(CommandName::Version) => Version::USAGE,
                Help::Command(CommandName::Fetch) => Fetch::USAGE,
                Help::Command(CommandName::Install) => Install::USAGE,
                Help::Command(CommandName::Shim) => Shim::USAGE,
            }
        );
        session.add_event_end(ActivityKind::Help, ExitCode::Success);
        Ok(true)
    }
}
