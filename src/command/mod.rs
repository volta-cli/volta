pub mod install;
pub mod uninstall;
pub mod current;
pub mod activate;
pub mod help;
pub mod version;

use docopt::{self, Docopt};
use console::style;
use std::process::exit;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const USAGE: &'static str = "
Notion: the hassle-free Node.js manager

Usage:
    notion [-v | --verbose] <command> [<args> ...]
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

#[derive(Debug, Deserialize)]
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

fn exit_with(e: docopt::Error) -> ! {
    // Docopt prints help messages to stdout but stderr is more traditional.
    if let Some(usage) = e.usage() {
        eprintln!("{}", usage);
        exit(0);
    }

    // Prefix fatal errors with a red, bold "error: " prefix.
    if e.fatal() {
        eprint!("{} ", style("error:").red().bold());
    }

    // Now let docopt do the rest.
    e.exit()
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
                    _ => {
                        Ok(args)
                    }
                }
            })
            .unwrap_or_else(|e| exit_with(e))
    }

    fn run(self) -> Result<(), docopt::Error> {
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
            Some(Command::Version) => {
                version::run(self.arg_args)
            }
            // This is a bit unpleasant but it's because docopt needs the
            // Command enum to be flat and parallel the set of subcommand,
            // even though it has special functionality for help commands.
            // So we already handled both the None and Some(Command::Help)
            // cases in Args::new(), but we can't prove that in the types.
            _ => { panic!("can't happen") }
        }
    }

}

pub fn run() -> ! {
    Args::new()
        .run()
        .unwrap_or_else(|e| exit_with(e));

    exit(0);
}
