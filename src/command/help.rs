use docopt::{self, Docopt};
use std::process::exit;
use console::style;

const USAGE: &'static str = "
Get some help with a notion command

Usage:
    notion help [<command>]
    notion help -h | --help

Options:
    -h, --help     Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<String>
}

pub fn throw<T>(usage: &str) -> Result<T, docopt::Error> {
    Err(docopt::Error::WithProgramUsage(Box::new(docopt::Error::Help), String::from(usage.trim())))
}

pub fn run(mut args: Vec<String>) -> Result<(), docopt::Error> {
    let mut argv = vec![String::from("notion"), String::from("help")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    let command = match args.arg_command {
        Some(command) => command,
        None => {
            return throw(super::USAGE)?;
        }
    };

    match &command[..] {
        "use" => {
            throw(super::activate::USAGE)?;
        }
        "current" => {
            throw(super::current::USAGE)?;
        }
        "help" => {
            throw(USAGE)?;
        }
        "version" => {
            throw(super::version::USAGE)?;
        }
        "install" => {
            throw(super::install::USAGE)?;
        }
        "uninstall" => {
            throw(super::uninstall::USAGE)?;
        }
        _ => {
            eprintln!("{} Unknown subcommand: '{}'", style("error:").red().bold(), command);
            exit(1);
        }
    }

    Ok(())
}
