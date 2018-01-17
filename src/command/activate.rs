use docopt::{self, Docopt};
use notion_core::{global, die};
use notion_core::version::Version;
use std::process::exit;

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
 
pub fn run(mut args: Vec<String>, _verbose: bool) -> Result<(), docopt::Error> {
    let mut argv = vec![String::from("notion"), String::from("use")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    if args.flag_global {
        // FIXME: compute the default
        let version = args.arg_version.unwrap();
        match global::set(Version::Public(version)) {
            Ok(_) => { }
            Err(err) => { die(err); }
        }
    } else {
        println!("not yet implemented; in the meantime you can modify your package.json.");
        exit(1);
    }

    Ok(())
}
