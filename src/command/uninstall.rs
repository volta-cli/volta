use docopt::Docopt;
use notion_core::session::Session;
use semver::Version;
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

    let version = Version::parse(&args.arg_version)?;
    let mut session = Session::new()?;
    session.catalog_mut()?.uninstall_node(&version)
}
