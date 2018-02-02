use docopt::Docopt;
use std::process::exit;
use notion_core::catalog::Catalog;
use notion_core::project::Project;
use notion_core::version::Version;
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

pub fn local() -> Result<Option<String>, failure::Error> {
    match Project::for_current_dir()? {
        Some(project) => {
            Ok(Some(project.lockfile()?.node.version.clone()))
        }
        None => Ok(None)
    }
}

pub fn global() -> Result<Option<String>, failure::Error> {
    let catalog = Catalog::current()?;
    Ok(catalog.node.map(|Version::Public(version)| version))
}

pub fn run(mut args: Vec<String>) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("current")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    if args.flag_local && !args.flag_global {
        match local()? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else if args.flag_global && !args.flag_local {
        match global()? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else {
        let (local, global) = (local()?, global()?);
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
