extern crate clap;
extern crate nodeup_core;

use std::io::Write;
use std::process::exit;

use clap::{Arg, ArgGroup, App, SubCommand};

use nodeup_core::current;

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
        Some("use")       => { not_yet_implemented("use"); }
        Some("current")   => {
            let submatches = matches.subcommand_matches("current").unwrap();
            let which = if submatches.is_present("local") {
                Some(current::Which::Local)
            } else if submatches.is_present("global") {
                Some(current::Which::Global)
            } else if submatches.is_present("system") {
                Some(current::Which::System)
            } else {
                None
            };

            match (which.is_some(), current::get(which)) {
                (true, Ok(Some(version))) => {
                    println!("v{}", version);
                }
                (true, Ok(None)) => {
                    exit(1);
                }
                (false, Ok(_)) => {
                    // FIXME: report on all three:
                    //   vx.y.z (local)
                    //   va.b.c (global)
                    unimplemented!()
                }
                (_, Err(err)) => {
                    nodeup_core::display_error(err);
                    exit(1);
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
