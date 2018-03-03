use notion_core::error::Fallible;

use {Notion, CliParseError};
use command::{Command, CommandName, Use, Version, Current, Install, Uninstall};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_command: Option<String>
}

pub(crate) enum Help {
    Notion,
    Command(CommandName)
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

    fn help() -> Self { Help::Command(CommandName::Help) }

    fn parse(_: Notion, Args { arg_command }: Args) -> Fallible<Help> {
        Ok(match arg_command {
            None => Help::Notion,
            Some(command) => {
                if let Ok(name) = command.parse() {
                    Help::Command(name)
                } else {
                    return Err(CliParseError {
                        usage: None,
                        error: format!("no such command: `{}`", command)
                    }.into());
                }
            }
        })
    }

    fn run(self) -> Fallible<bool> {
        eprintln!("{}", match self {
            Help::Notion                          => Notion::USAGE,
            Help::Command(CommandName::Use)       => Use::USAGE,
            Help::Command(CommandName::Current)   => Current::USAGE,
            Help::Command(CommandName::Help)      => Help::USAGE,
            Help::Command(CommandName::Version)   => Version::USAGE,
            Help::Command(CommandName::Install)   => Install::USAGE,
            Help::Command(CommandName::Uninstall) => Uninstall::USAGE
        });
        Ok(true)
    }
}
