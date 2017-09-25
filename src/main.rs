extern crate clap;

use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("Nemo")
        .version("1.0")
        .about("The Node toolchain manager")

        // nemo install [version]
        .subcommand(SubCommand::with_name("install")
            .about("install the toolchain to the local machine")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(false)))

        // nemo use [version]
        .subcommand(SubCommand::with_name("use")
            .about("activate a particular toolchain version")
            .arg(Arg::with_name("version")
                .help("Node.js version specifier")
                .required(false)))

        // nemo current
        .subcommand(SubCommand::with_name("current")
            .about("display the current activated toolchain version"))

        // nemo version
        .subcommand(SubCommand::with_name("version")
            .about("display the Nemo version"))

        // nemo help
        .subcommand(SubCommand::with_name("help")
            .about("display help information"))
        .get_matches();

    match matches.subcommand_name() {
        Some("install") => { println!("install!"); }
        Some("use") => { println!("use!"); }
        Some("current") => { println!("current!"); }
        Some("version") => { println!("version!"); }
        Some("help") => { println!("help!"); }
        _ => { println!("who knows?"); }
    }
}
