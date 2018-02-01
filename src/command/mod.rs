pub mod install;
pub mod uninstall;
pub mod current;
pub mod activate;
pub mod help;
pub mod version;

use docopt::{self, Docopt};
use std::process::exit;
use notion_core::style::{display_error, display_error_prefix};
use failure;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const USAGE: &'static str = "
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
    current        Display the currently activated toolchain version
    help           Display this message
    version        Print version info and exit

See 'notion help <command>' for more information on a specific command.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<Command>,
    arg_args: Vec<String>,
    flag_version: bool,
    flag_verbose: bool
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum Command {
    Install,
    Uninstall,
    Use,
    Current,
    Help,
    Version
}

trait Usage {
    fn usage(&self) -> Option<&str>;
}

impl Usage for docopt::Error {

    fn usage(&self) -> Option<&str> {
        match *self {
            docopt::Error::WithProgramUsage(ref cause, ref usage) => {
                match **cause {
                    docopt::Error::Help => Some(usage),
                    _ => None
                }
            }
            docopt::Error::Help => {
                Some(USAGE)
            }
            _ => None
        }
    }

}

trait Die {
    fn die(self) -> !;
}

impl Die for docopt::Error {

    fn die(self) -> ! {
        // Docopt prints help messages to stdout but stderr is more traditional.
        if let Some(usage) = self.usage() {
            eprintln!("{}", usage);
            exit(0);
        }

        // Prefix fatal errors with a red, bold "error: " prefix.
        if self.fatal() {
            display_error_prefix();
        }

        // Now let docopt do the rest.
        self.exit()
    }

}

impl Die for failure::Error {

    fn die(self) -> ! {
        display_error(self);
        exit(1);
    }

}

impl Args {

    fn new() -> Args {
        Docopt::new(USAGE)
            .and_then(|d| d.options_first(true).version(Some(String::from(VERSION))).deserialize())
            .and_then(|args: Args| -> Result<Args, docopt::Error> {
                match args.arg_command {
                    None => {
                        return help::throw(USAGE);
                    }
                    Some(Command::Help) => {
                        help::run(args.arg_args)?;
                        exit(0);
                    }
                    Some(Command::Version) => {
                        version::run(args.arg_args)?;
                        exit(0);
                    }
                    _ => {
                        Ok(args)
                    }
                }
            })
            .unwrap_or_else(|e| e.die())
    }

    fn run(self) -> Result<(), failure::Error> {
        match self.arg_command {
            Some(Command::Install) => {
                install::run(self.arg_args, self.flag_verbose)
            }
            Some(Command::Uninstall) => {
                uninstall::run(self.arg_args, self.flag_verbose)
            }
            Some(Command::Use) => {
                activate::run(self.arg_args, self.flag_verbose)
            }
            Some(Command::Current) => {
                current::run(self.arg_args)
            }
            // This is a bit unpleasant but it's because docopt needs the
            // Command enum to be flat and parallel the set of subcommand,
            // even though it has special built-in functionality for help
            // and version commands.
            //
            // So we already handled the None, Some(Comand::Version), and
            // Some(Command::Help) cases in Args::new(), but the types do
            // not prove this.
            _ => { panic!("can't happen") }
        }
    }

}

pub fn run() -> ! {
    Args::new()
        .run()
        .unwrap_or_else(|e| e.die());

    exit(0);
}
