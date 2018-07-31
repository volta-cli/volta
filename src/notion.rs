extern crate console;
extern crate docopt;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate notion_core;
#[macro_use]
extern crate notion_fail;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod command;
mod error;

use std::process::exit;
use std::string::ToString;

use docopt::Docopt;

use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, display_unknown_error, ErrorContext};
use notion_fail::{FailExt, Fallible, NotionError};

use command::{Command, CommandName, Config, Current, Deactivate, Default, Help, Install, Shim,
              Uninstall, Use, Version};
use error::{CliParseError, CommandUnimplementedError, DocoptExt, NotionErrorExt};

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
Notion: the hassle-free Node.js manager

Usage:
    notion [-v | --verbose] [<command> <args> ...]
    notion -h | --help
    notion -V | --version

Options:
    -h, --help     Display this message
    -V, --version  Print version info and exit
    -v, --verbose  Use verbose output

Some common notion commands are:
    install        Install a toolchain to the local machine
    uninstall      Uninstall a toolchain from the local machine
    use            Activate a particular toolchain version
    config         Get or set configuration values
    current        Display the currently activated toolchain version
    deactivate     Remove Notion from the current shell
    default        Get or set the default toolchain version
    shim           View and manage shims
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

    fn go(session: &mut Session) -> Fallible<bool> {
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
                    throw!(CliParseError {
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
                    throw!(err.with_context(CliParseError::from_docopt));
                }
            }
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        match self.command {
            CommandName::Install => Install::go(self, session),
            CommandName::Uninstall => Uninstall::go(self, session),
            CommandName::Use => Use::go(self, session),
            CommandName::Config => Config::go(self, session),
            CommandName::Current => Current::go(self, session),
            CommandName::Deactivate => Deactivate::go(self, session),
            CommandName::Default => Default::go(self, session),
            CommandName::Shim => Shim::go(self, session),
            CommandName::Help => Help::go(self, session),
            CommandName::Version => Version::go(self, session),
        }
    }
}

fn display_error_and_usage(err: &NotionError) {
    if err.is_user_friendly() {
        display_error(ErrorContext::Notion, err);
    } else {
        display_unknown_error(ErrorContext::Notion, err);
    }

    if let Some(ref usage) = err.usage() {
        eprintln!();
        eprintln!("{}", usage);
    }
}

/// The entry point for the `notion` CLI.
pub fn main() {
    let mut session = match Session::new() {
        Ok(session) => session,
        Err(err) => {
            display_error_and_usage(&err);
            exit(1);
        }
    };

    session.add_event_start(ActivityKind::Notion);

    let exit_code = match Notion::go(&mut session) {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(err) => {
            display_error_and_usage(&err);
            session.add_event_error(ActivityKind::Notion, &err);
            err.exit_code()
        }
    };
    session.add_event_end(ActivityKind::Notion, exit_code);
    session.exit(exit_code);
}
