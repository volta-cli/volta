mod activate;
mod config;
mod current;
mod deactivate;
mod fetch;
mod help;
mod install;
mod pin;
mod use_;
mod version;

pub(crate) use self::activate::Activate;
pub(crate) use self::config::Config;
pub(crate) use self::current::Current;
pub(crate) use self::deactivate::Deactivate;
pub(crate) use self::fetch::Fetch;
pub(crate) use self::help::Help;
pub(crate) use self::install::Install;
pub(crate) use self::pin::Pin;
pub(crate) use self::use_::Use;
pub(crate) use self::version::Version;

use docopt::Docopt;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use notion_core::session::Session;
use notion_fail::{throw, FailExt, Fallible};

use crate::error::from_docopt_error;
use crate::{DocoptExt, Notion};

use std::fmt::{self, Display};
use std::str::FromStr;

/// Represents the set of Notion command names.
#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) enum CommandName {
    Fetch,
    Install,
    Pin,
    Use,
    Config,
    Current,
    Deactivate,
    Activate,
    Help,
    Version,
}

impl Display for CommandName {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match *self {
                CommandName::Fetch => "fetch",
                CommandName::Install => "install",
                CommandName::Pin => "pin",
                CommandName::Use => "use",
                CommandName::Config => "config",
                CommandName::Deactivate => "deactivate",
                CommandName::Activate => "activate",
                CommandName::Current => "current",
                CommandName::Help => "help",
                CommandName::Version => "version",
            }
        )
    }
}

impl FromStr for CommandName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fetch" => CommandName::Fetch,
            "install" => CommandName::Install,
            "pin" => CommandName::Pin,
            "use" => CommandName::Use,
            "config" => CommandName::Config,
            "current" => CommandName::Current,
            "deactivate" => CommandName::Deactivate,
            "activate" => CommandName::Activate,
            "help" => CommandName::Help,
            "version" => CommandName::Version,
            _ => {
                throw!(());
            }
        })
    }
}

/// A Notion command.
pub(crate) trait Command: Sized {
    /// The intermediate type Docopt should deserialize the parsed command into.
    type Args: DeserializeOwned;

    /// The full usage documentation for this command. This can contain leading and trailing
    /// whitespace, which will be trimmed before printing to the console.
    const USAGE: &'static str;

    /// Produces a variant of this type representing the `notion <command> --help`
    /// option.
    fn help() -> Self;

    /// Parses the intermediate deserialized arguments into the full command.
    fn parse(notion: Notion, args: Self::Args) -> Fallible<Self>;

    /// Executes the command. Returns `Ok(true)` if the process should return 0,
    /// `Ok(false)` if the process should return 1, and `Err(e)` if the process
    /// should return `e.exit_code()`.
    fn run(self, session: &mut Session) -> Fallible<()>;

    /// Top-level convenience method for taking a Notion invocation and executing
    /// this command with the arguments taken from the Notion invocation.
    fn go(notion: Notion, session: &mut Session) -> Fallible<()> {
        let argv = notion.full_argv();
        let args = Docopt::new(Self::USAGE).and_then(|d| d.argv(argv).deserialize());

        match args {
            Ok(args) => Self::parse(notion, args)?.run(session),
            Err(err) => {
                // Docopt models `-h` and `--help` as errors, so this
                // normalizes them to a normal `notion help` command.
                if err.is_help() {
                    Self::help().run(session)
                }
                // Otherwise it's a true docopt error, so rethrow it.
                else {
                    throw!(err.with_context(from_docopt_error));
                }
            }
        }
    }
}
