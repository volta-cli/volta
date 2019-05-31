use std::str::FromStr;

use structopt::StructOpt;

use volta_core::session::{ActivityKind, Session};
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct List {
    /// Display
    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,

    /// Display in a human-friendly format. (This is the default for TTYs.)
    #[structopt(short = "h", long = "human", conflicts_with = "plain")]
    human: bool,

    /// Display in a human-friendly format. (This is the default for non-TTYs.)
    #[structopt(short = "p", long = "plain", conflicts_with = "human")]
    plain: bool,
}

enum Format {
    Human,
    Plain,
}

#[derive(StructOpt)]
enum Subcommand {
    /// Show every item in the toolchain.
    #[structopt(name = "all")]
    All,

    /// Show locally cached Node versions.
    #[structopt(name = "node")]
    Node,

    /// Show locally cached Yarn versions.
    #[structopt(name = "yarn")]
    Yarn,

    /// Show locally cached versions of a package or a package binary.
    #[structopt(name = "<package or tool>")]
    PackageOrTool { name: String },
}

impl FromStr for Subcommand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "all" => Subcommand::All,
            "node" => Subcommand::Node,
            "yarn" => Subcommand::Yarn,
            s => Subcommand::PackageOrTool { name: s.into() },
        })
    }
}

impl List {
    fn format(&self) -> Format {
        // We start by checking if the user has explicitly set a value: if they
        // have, that trumps our TTY-checking. Then, if the user has *not*
        // specified an option, we use `Human` mode for TTYs and `Plain` for
        // non-TTY contexts.
        if self.human {
            Format::Human
        } else if self.plain {
            Format::Plain
        } else if atty::is(atty::Stream::Stdout) {
            Format::Human
        } else {
            Format::Plain
        }
    }
}

impl Command for List {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::List);
        let format = self.format();

        let toolchain_to_display = match self.subcommand {
            Some(Subcommand::All) => unimplemented!(),
            Some(Subcommand::Node) => unimplemented!(),
            Some(Subcommand::Yarn) => unimplemented!(),
            Some(Subcommand::PackageOrTool { name }) => unimplemented!(),
            None => unimplemented!(),
        };

        session.add_event_end(ActivityKind::List, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
