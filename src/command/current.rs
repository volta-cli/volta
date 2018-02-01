use docopt::Docopt;
use std::process::exit;
use notion_core::current;
use failure;

pub const USAGE: &'static str = "
Display the currently activated toolchain

Usage:
    notion current [options]

Options:
    -h, --help     Display this message
    -l, --local    Display local toolchain
    -g, --global   Display global toolchain
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_local: bool,
    flag_global: bool
}

pub fn run(mut args: Vec<String>) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("current")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    if args.flag_local && !args.flag_global {
        match current::local()? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else if args.flag_global && !args.flag_local {
        match current::global()? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else {
        let (local, global) = current::both()?;
        let global_active = local.is_none() && global.is_some();
        let none = local.is_none() && global.is_none();
        // FIXME: abstract this
        for version in local {
            println!("local: v{} (active)", version);
        }
        for version in global {
            println!("global: v{}{}", version, if global_active { " (active)" } else { "" });
        }
        if none {
            exit(1);
        }
    }

    Ok(())
}
