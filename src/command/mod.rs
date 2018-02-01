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

fn docopt_help_error(e: &docopt::Error) -> Option<&str> {
    match e {
        &docopt::Error::WithProgramUsage(ref err, ref usage) => {
            if let &docopt::Error::Help = &**err {
                Some(usage)
            } else {
                None
            }
        }
        &docopt::Error::Help => {
            Some(USAGE)
        }
        _ => None
    }
}

fn exit_with(e: docopt::Error) -> ! {
    // Docopt prints help messages to stdout but stderr is more traditional.
    if let Some(usage) = docopt_help_error(&e) {
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

pub fn run() {
    Docopt::new(USAGE)
        .and_then(|d| d.options_first(true).version(Some(String::from(VERSION))).deserialize())
        .and_then(|args: Args| {
            match args.arg_command {
                None => {
                    help::throw(USAGE)
                }
                Some(Command::Install) => {
                    install::run(args.arg_args, args.flag_verbose)
                }
                Some(Command::Uninstall) => {
                    uninstall::run(args.arg_args, args.flag_verbose)
                }
                Some(Command::Use) => {
                    activate::run(args.arg_args, args.flag_verbose)
                }
                Some(Command::Current) => {
                    current::run(args.arg_args)
                }
                Some(Command::Help) => {
                    help::run(args.arg_args)
                }
                Some(Command::Version) => {
                    version::run(args.arg_args)
                }
            }
        })
        .unwrap_or_else(|e| exit_with(e));

}
