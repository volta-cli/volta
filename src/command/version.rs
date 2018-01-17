use docopt::{self, Docopt};

pub const USAGE: &'static str = "
Display version information

Usage:
    notion version
    notion version -h | --help

Options:
    -h, --help     Display this message
";

#[derive(Debug, Deserialize)]
struct Args;

pub fn run(mut args: Vec<String>) -> Result<(), docopt::Error> {
    let mut argv = vec![String::from("notion"), String::from("version")];
    argv.append(&mut args);

    let _: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    return Err(docopt::Error::Version(String::from(super::VERSION)));
}
