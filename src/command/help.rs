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

pub fn run(mut args: Vec<String>) -> Result<(), docopt::Error> {
    let mut argv = vec![String::from("notion"), String::from("help")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    let command = match args.arg_command {
        Some(command) => command,
        None => {
            super::throw_help(super::USAGE)?;
            panic!("can't happen")
        }
    };

    match &command[..] {
        "use" => {
            super::throw_help(super::activate::USAGE)?;
        }
        "current" => {
            super::throw_help(super::current::USAGE)?;
        }
        "help" => {
            super::throw_help(USAGE)?;
        }
        "version" => {
            super::throw_help(super::version::USAGE)?;
        }
        "install" => {
            super::throw_help(super::install::USAGE)?;
        }
        "uninstall" => {
            super::throw_help(super::uninstall::USAGE)?;
        }
        _ => {
            eprintln!("{} Unknown subcommand: '{}'", style("error:").red().bold(), command);
            exit(1);
        }
    }

    Ok(())
}
