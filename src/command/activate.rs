use docopt::Docopt;
use notion_core::global;
use notion_core::version::Version;
use std::process::exit;
use failure;

pub const USAGE: &'static str = "
Activate a particular toolchain version

Usage:
    notion use [options] <version>
    notion use -h | --help

Options:
    -h, --help     Display this message
    -g, --global   Activate the toolchain globally
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_version: Option<String>,
    flag_global: bool
}

pub fn run(mut args: Vec<String>, _verbose: bool) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("use")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    if args.flag_global {
        // FIXME: compute the default
        let version = args.arg_version.unwrap();
        global::set(Version::Public(version))?;
    } else {
        println!("not yet implemented; in the meantime you can modify your package.json.");
        exit(1);
    }

    Ok(())
}
