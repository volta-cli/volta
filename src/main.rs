extern crate clap;
extern crate nodeup_core;

use std::io::Write;
use std::process::exit;

use clap::{Arg, ArgGroup, App, SubCommand};

use nodeup_core::{current, die};
use nodeup_core::global;
use nodeup_core::version::Version;

fn main() {
    let app = App::new("nodeup")
        .version("0.1")
        .about("The Node toolchain manager")

        // nodeup install [version]
        .subcommand(SubCommand::with_name("install")
            .about("install a toolchain to the local machine")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(true)))

        // nodeup uninstall version
        .subcommand(SubCommand::with_name("uninstall")
            .about("uninstall a toolchain from the local machine")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(true)))

        // nodeup use [version]
        .subcommand(SubCommand::with_name("use")
            .about("activate a particular toolchain version")
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .help("")
                .required(false))
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(false)))

        // nodeup current
        .subcommand(SubCommand::with_name("current")
            .about("display the currently activated toolchain version")
            .arg(Arg::with_name("local")
                .short("l")
                .long("local")
                .help("")
                .required(false))
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .help("")
                .required(false))
            .arg(Arg::with_name("system")
                .short("s")
                .long("system")
                .help("")
                .required(false))
            .group(ArgGroup::with_name("current_kind")
                .args(&["local", "global", "system"])
                .required(false)))

        // nodeup version
        .subcommand(SubCommand::with_name("version")
            .about("display the nodeup version"))

        // nodeup help
        .subcommand(SubCommand::with_name("help")
            .about("display help information"));

    let mut help_bytes: Vec<u8> = Vec::new();
    app.write_help(&mut help_bytes).unwrap();

    let matches = app.get_matches();
    match matches.subcommand_name() {
        Some("install")   => {
            let submatches = matches.subcommand_matches("install").unwrap();
            let version = submatches.value_of("version").unwrap();
            if let Err(err) = nodeup_core::install::by_version(&version) {
                nodeup_core::display_error(err);
                exit(1);
            }
        }
        Some("uninstall") => {
            let submatches = matches.subcommand_matches("uninstall").unwrap();
            let version = submatches.value_of("version").unwrap();
            if let Err(err) = nodeup_core::uninstall::by_version(&version) {
                nodeup_core::display_error(err);
                exit(1);
            }
        }
        Some("use")       => {
            let submatches = matches.subcommand_matches("use").unwrap();
            if submatches.is_present("global") {
                match global::set(Version::Public(String::from(submatches.value_of("version").unwrap()))) {
                    Ok(_) => { }
                    Err(err) => { die(err); }
                }
            } else {
                println!("not yet implemented; in the meantime you can modify your package.json.");
                exit(1);
            }
        }
        Some("current")   => {
            let submatches = matches.subcommand_matches("current").unwrap();
            // FIXME: abstract the bodies here
            if submatches.is_present("local") {
                match current::local() {
                    Ok(Some(version)) => { println!("v{}", version); }
                    Ok(None)          => { exit(1); }
                    Err(err)          => { die(err); }
                }
            } else if submatches.is_present("global") {
                match current::global() {
                    Ok(Some(version)) => { println!("v{}", version); }
                    Ok(None)          => { exit(1); }
                    Err(err)          => { die(err); }
                }
            } else {
                match current::both() {
                    Ok((local, global)) => {
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
                    Err(err) => { die(err); }
                }
            }
        }
        Some("version")   => { not_yet_implemented("version"); }
        Some("help")      => { help(&help_bytes); }
        Some(_)           => { panic!("internal error (command parser)"); }
        None              => { help(&help_bytes); }
    }
}

fn help(help_bytes: &[u8]) {
    let mut stderr = ::std::io::stderr();
    stderr.write_all(help_bytes).unwrap();
    eprintln!();
}

fn not_yet_implemented(command: &str) {
    panic!("command '{}' not yet implemented", command)
}
