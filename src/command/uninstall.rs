use docopt::Docopt;
use std::process::exit;
use notion_core;
use failure;

pub const USAGE: &'static str = "
Uninstall a toolchain from the local machine

Usage:
    notion uninstall <version>
    notion uninstall -h | --help

Options:
    -h, --help     Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_version: String
}

pub fn run(mut args: Vec<String>, _verbose: bool) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("uninstall")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    if let Err(err) = notion_core::uninstall::by_version(&args.arg_version) {
        notion_core::style::display_error(err);
        exit(1);
    }

    Ok(())
}
