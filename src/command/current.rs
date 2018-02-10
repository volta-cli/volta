use docopt::Docopt;
use std::process::exit;
use std::string::ToString;
use notion_core::session::Session;
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

pub fn local(session: &Session) -> Result<Option<String>, failure::Error> {
    let project = session.project();
    let project = match project {
        Some(ref project) => project,
        None => { return Ok(None); }
    };

    let req = project.manifest().node_req();
    let catalog = session.catalog()?;
    Ok(catalog.node.resolve_local(&req).map(|v| v.to_string()))
}

pub fn global(session: &Session) -> Result<Option<String>, failure::Error> {
    let catalog = session.catalog()?;
    Ok(catalog.node.current.clone().map(|v| v.to_string()))
}

pub fn run(mut args: Vec<String>) -> Result<(), failure::Error> {
    let mut argv = vec![String::from("notion"), String::from("current")];
    argv.append(&mut args);

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv).deserialize())?;

    let session = Session::new()?;

    if args.flag_local && !args.flag_global {
        match local(&session)? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else if args.flag_global && !args.flag_local {
        match global(&session)? {
            Some(version) => { println!("v{}", version); }
            None          => { exit(1); }
        }
    } else {
        let (local, global) = (local(&session)?, global(&session)?);
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
