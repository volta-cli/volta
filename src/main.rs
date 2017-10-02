extern crate clap;
extern crate flate2;
extern crate tar;
extern crate indicatif;
extern crate term_size;
extern crate reqwest;
extern crate toml;

mod config;
mod provision;
mod install;
mod uninstall;

use std::io::Write;

use clap::{Arg, App, SubCommand};

use config::{Config, Version};

fn main() {
    let app = App::new("Nemo")
        .version("0.1")
        .about("The Node toolchain manager")

        // nemo install [version]
        .subcommand(SubCommand::with_name("install")
            .about("install a toolchain to the local machine")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(true)))

        // nemo uninstall version
        .subcommand(SubCommand::with_name("uninstall")
            .about("uninstall a toolchain from the local machine")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(true)))

        // nemo use [version]
        .subcommand(SubCommand::with_name("use")
            .about("activate a particular toolchain version")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(false)))

        // nemo local
        .subcommand(SubCommand::with_name("local")
            .about("display the toolchain version associated with the local project"))

        // nemo current
        .subcommand(SubCommand::with_name("current")
            .about("display the current activated toolchain version"))

        // nemo version
        .subcommand(SubCommand::with_name("version")
            .about("display the Nemo version"))

        // nemo help
        .subcommand(SubCommand::with_name("help")
            .about("display help information"));

    let mut help_bytes: Vec<u8> = Vec::new();
    app.write_help(&mut help_bytes).unwrap();

    let matches = app.get_matches();
    match matches.subcommand_name() {
        Some("install")   => {
            let submatches = matches.subcommand_matches("install").unwrap();
            let version = submatches.value_of("version").unwrap();
            install::by_version(&version);
        }
        Some("uninstall") => {
            let submatches = matches.subcommand_matches("uninstall").unwrap();
            let version = submatches.value_of("version").unwrap();
            uninstall::by_version(&version);
        }
        Some("local")     => {
            let Config { node: Version::Public(version) } = config::read().unwrap();
            println!("v{}", version);
        }
        Some("use")       => { not_yet_implemented("use"); }
        Some("current")   => { not_yet_implemented("current"); }
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