use std::fmt::{self, Display};
use std::str::FromStr;

use docopt::Docopt;
use serde::Deserialize;

use notion_core::session::Session;
use notion_fail::{FailExt, Fallible};

use Notion;
use command::{Command, CommandName, Help};

use CliParseError;

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_command: SubcommandName,
    arg_args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) enum SubcommandName {
    Get,
    Set,
    Delete,
    List,
    Edit,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Key {
    arg_key: String
}

#[derive(Debug, Deserialize)]
pub(crate) struct KeyValue {
    arg_key: String,
    arg_value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Nullary;

impl Display for SubcommandName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                SubcommandName::Get => "get",
                SubcommandName::Set => "set",
                SubcommandName::Delete => "delete",
                SubcommandName::List => "list",
                SubcommandName::Edit => "edit",
            }
        )
    }
}

impl FromStr for SubcommandName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "get" => SubcommandName::Get,
            "set" => SubcommandName::Set,
            "delete" => SubcommandName::Delete,
            "list" => SubcommandName::List,
            "edit" => SubcommandName::Edit,
            _ => {
                throw!(());
            }
        })
    }
}

pub(crate) enum Config {
    Help,
    Subcommand(Subcommand),
}

pub(crate) enum Subcommand {
    Get {
        // Not yet implemented.
        #[allow(dead_code)]
        key: String
    },
    Set {
        // Not yet implemented.
        #[allow(dead_code)]
        key: String,

        // Not yet implemented.
        #[allow(dead_code)]
        value: String
    },
    Delete {
        // Not yet implemented.
        #[allow(dead_code)]
        key: String
    },
    List,
    Edit,
}

fn parse_subcommand<'de, T: Deserialize<'de>>(subcommand: &str, usage: &str, mut args: Vec<String>) -> Fallible<T> {
    let mut argv = vec!["notion".to_string(), "config".to_string(), subcommand.to_string()];
    argv.append(&mut args);
    let usage = format!("Usage: notion config {} {}", subcommand, usage);
    Docopt::new(&usage[..]).and_then(|d| d.argv(argv).deserialize())
        .map_err(|err| { err.with_context(CliParseError::from_docopt) })
}

impl Command for Config {
    type Args = Args;

    const USAGE: &'static str = "
Get or set configuration values

Usage:
    notion config <command> [<args> ...]
    notion config -h | --help

Options:
    -h, --help     Display this message

Config commands:
    get <key>
    set <key> <value>
    delete <key>
    list
    edit
";

    fn help() -> Self {
        Config::Help
    }

    fn parse(_: Notion, args: Args) -> Fallible<Config> {
        let command = args.arg_command;
        let argv = args.arg_args.unwrap_or_else(|| vec![]);
        Ok(match command {
            SubcommandName::Get => {
                let Key { arg_key } = parse_subcommand("get", "<key>", argv)?;
                Config::Subcommand(Subcommand::Get { key: arg_key })
            },
            SubcommandName::Set => {
                let KeyValue { arg_key, arg_value } = parse_subcommand("set", "<key> <value>", argv)?;
                Config::Subcommand(Subcommand::Set { key: arg_key, value: arg_value })
            },
            SubcommandName::Delete => {
                let Key { arg_key } = parse_subcommand("delete", "<key>", argv)?;
                Config::Subcommand(Subcommand::Delete { key: arg_key })
            },
            SubcommandName::List => {
                let Nullary = parse_subcommand("list", "", argv)?;
                Config::Subcommand(Subcommand::List)
            },
            SubcommandName::Edit => {
                let Nullary = parse_subcommand("edit", "", argv)?;
                Config::Subcommand(Subcommand::Edit)
            }
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        //session.add_event_start(ActivityKind::Version);
        let result = match self {
            Config::Help => Help::Command(CommandName::Config).run(session),
            Config::Subcommand(Subcommand::Get { key: _ }) => {
                Ok(true)
            }
            Config::Subcommand(Subcommand::Set { key: _, value: _ }) => {
                unimplemented!()
            }
            Config::Subcommand(Subcommand::Delete { key: _ }) => {
                unimplemented!()
            }
            Config::Subcommand(Subcommand::List) => {
                unimplemented!()
            }
            Config::Subcommand(Subcommand::Edit) => {
                unimplemented!()
            }
        };
        //session.add_event_end(ActivityKind::Version, 0);
        result
    }
}
