use docopt;
mod command;
mod error;

use std::string::ToString;

use docopt::Docopt;
use serde::Deserialize;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, ErrorContext};
use notion_fail::{throw, ExitCode, FailExt, Fallible, NotionError};

use crate::error::{cli_parse_error, DocoptExt, NotionErrorExt};
use command::{
    Activate, Command, CommandName, Config, Current, Deactivate, Fetch, Help, Install, Pin, Use,
    Version,
};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<CommandName>,
    arg_args: Vec<String>,
    flag_version: bool,
    flag_verbose: bool,
}

pub(crate) struct Notion {
    command: CommandName,
    args: Vec<String>,
    verbose: bool,
}

impl Notion {
    pub(crate) const USAGE: &'static str = "
Notion: the hassle-free JavaScript toolchain manager

Usage:
    notion [-v | --verbose] [<command> <args> ...]
    notion -h | --help
    notion -V | --version

Options:
    -h, --help     Display this message
    -V, --version  Print version info and exit
    -v, --verbose  Use verbose output

Some common notion commands are:
    fetch          Fetch a tool to the local machine
    install        Install a tool in the user toolchain
    pin            Select a tool for the current project's toolchain
    config         Get or set configuration values
    current        Display the currently activated Node version
    deactivate     Disable Notion in the current shell
    activate       Re-Enable Notion in the current shell
    help           Display this message
    version        Print version info and exit

See 'notion help <command>' for more information on a specific command.
";

    // This isn't used yet but we can use it for verbose mode in the future.
    #[allow(dead_code)]
    pub(crate) fn verbose(&self) -> bool {
        self.verbose
    }

    pub(crate) fn full_argv(&self) -> Vec<String> {
        let mut argv = vec![String::from("notion"), self.command.to_string()];
        let mut sub_argv = self.args.clone();
        argv.append(&mut sub_argv);
        argv
    }

    fn go(session: &mut Session) -> Fallible<()> {
        Self::parse()?.run(session)
    }

    fn parse() -> Fallible<Notion> {
        let mut command_string: Option<String> = None;

        let args: Result<Args, docopt::Error> = Docopt::new(Notion::USAGE).and_then(|d| {
            d.options_first(true)
                .version(Some(String::from(VERSION)))
                .parse()
                .and_then(|vals| {
                    {
                        // Save the value of the <command> argument for error reporting.
                        let command = vals.get_str("<command>");
                        if command != "" {
                            command_string = Some(command.to_string());
                        }
                    }
                    vals.deserialize()
                })
        });

        Ok(match args {
            // Normalize the default `notion` command as `notion help`.
            Ok(Args {
                arg_command: None, ..
            }) => Notion {
                command: CommandName::Help,
                args: vec![],
                verbose: false,
            },

            Ok(Args {
                arg_command: Some(cmd),
                arg_args,
                flag_verbose,
                ..
            }) => Notion {
                command: cmd,
                args: arg_args,
                verbose: flag_verbose,
            },

            Err(err) => {
                // Docopt models `-h` and `--help` as errors, so this
                // normalizes them to a normal `notion help` command.
                if err.is_help() {
                    Notion {
                        command: CommandName::Help,
                        args: vec![],
                        verbose: false,
                    }
                }
                // Docopt models `-V` and `--version` as errors, so this
                // normalizes them to a normal `notion version` command.
                else if err.is_version() {
                    Notion {
                        command: CommandName::Version,
                        args: vec![],
                        verbose: false,
                    }
                }
                // The only type that gets deserialized is CommandName. If
                // the command name is not one of the expected set, we get
                // an Error::Deserialize.
                else if let docopt::Error::Deserialize(_) = err {
                    throw!(ErrorDetails::CliParseError {
                        usage: None,
                        error: if let Some(command) = command_string {
                            format!("no such command: `{}`", command)
                        } else {
                            format!("invalid command")
                        },
                    });
                }
                // Otherwise the other docopt error messages are pretty
                // reasonable, so just wrap and then rethrow.
                else {
                    throw!(err.with_context(cli_parse_error));
                }
            }
        })
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        match self.command {
            CommandName::Fetch => Fetch::go(self, session),
            CommandName::Install => Install::go(self, session),
            CommandName::Pin => Pin::go(self, session),
            CommandName::Use => Use::go(self, session),
            CommandName::Config => Config::go(self, session),
            CommandName::Current => Current::go(self, session),
            CommandName::Deactivate => Deactivate::go(self, session),
            CommandName::Activate => Activate::go(self, session),
            CommandName::Help => Help::go(self, session),
            CommandName::Version => Version::go(self, session),
        }
    }
}

fn display_error_and_usage(err: &NotionError) {
    display_error(ErrorContext::Notion, err);

    if let Some(ref usage) = err.usage() {
        eprintln!();
        eprintln!("{}", usage);
    }
}

/// The entry point for the `notion` CLI.
pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Notion);

    let exit_code = match Notion::go(&mut session) {
        Ok(_) => ExitCode::Success,
        Err(err) => {
            display_error_and_usage(&err);
            session.add_event_error(ActivityKind::Notion, &err);
            err.exit_code()
        }
    };
    session.add_event_end(ActivityKind::Notion, exit_code);
    session.exit(exit_code);
}
