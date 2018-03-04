mod install;
mod uninstall;
mod current;
mod use_;
mod help;
mod version;

pub(crate) use self::install::Install;
pub(crate) use self::uninstall::Uninstall;
pub(crate) use self::current::Current;
pub(crate) use self::use_::Use;
pub(crate) use self::help::Help;
pub(crate) use self::version::Version;

use docopt::Docopt;
use serde::de::DeserializeOwned;

use notion_fail::{FailExt, Fallible};

use {Notion, DocoptExt, CliParseError};

use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) enum CommandName {
    Install,
    Uninstall,
    Use,
    Current,
    Help,
    Version
}

impl Display for CommandName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", match *self {
            CommandName::Install   => "install",
            CommandName::Uninstall => "uninstall",
            CommandName::Use       => "use",
            CommandName::Current   => "current",
            CommandName::Help      => "help",
            CommandName::Version   => "version"
        })
    }
}

impl FromStr for CommandName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "install"   => CommandName::Install,
            "uninstall" => CommandName::Uninstall,
            "use"       => CommandName::Use,
            "current"   => CommandName::Current,
            "help"      => CommandName::Help,
            "version"   => CommandName::Version,
            _ => {
                throw!(());
            }
        })
    }
}

pub(crate) trait Command: Sized {
    type Args: DeserializeOwned;

    const USAGE: &'static str;

    fn help() -> Self;

    fn parse(notion: Notion, args: Self::Args) -> Fallible<Self>;

    fn run(self) -> Fallible<bool>;

    fn go(notion: Notion) -> Fallible<bool> {
        let argv = notion.full_argv();
        let args = Docopt::new(Self::USAGE)
            .and_then(|d| d.argv(argv).deserialize());

        match args {
            Ok(args) => {
                Self::parse(notion, args)?.run()
            }
            Err(err) => {
                // Docopt models `-h` and `--help` as errors, so this
                // normalizes them to a normal `notion help` command.
                if err.is_help() {
                    Self::help().run()
                }

                // Otherwise it's a true docopt error, so rethrow it.
                else {
                    throw!(err.with_context(CliParseError::from_docopt));
                }
            }
        }
    }
}
