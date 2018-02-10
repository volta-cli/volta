use docopt::Docopt;
use notion_core::catalog::Catalog;
use notion_core::session::Session;
use failure;
use semver::VersionReq;

pub const USAGE: &'static str = "
Install a toolchain to the local machine

Usage:
    notion install <version>
    notion install -h | --help

Options:
    -h, --help     Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_version: String
}

pub fn run(mut args: Vec<String>, _verbose: bool) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("install")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    let mut session = Session::new()?;
    session.install_node(&VersionReq::parse(&args.arg_version)?)?;

    Ok(())
}
