mod human;
mod plain;


use std::path::PathBuf;
use std::str::FromStr;

use semver::Version;
use structopt::StructOpt;
use volta_core::session::{ActivityKind, Session};
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(Copy, Clone)]
enum Format {
    Human,
    Plain,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Format::Human),
            "plain" => Ok(Format::Plain),
            _ => Err("No".into()),
        }
    }
}

enum Source {
    Project(PathBuf),
    User,
    None,
}

struct Package {
    pub name: String,
    pub version: Version,
}

struct ToolHost {
    pub package: Package,
    pub node: Version,
}

struct Tool {
    name: String,
    host: ToolHost,
}

struct Node {
    source: Source,
    version: Version,
}

enum PackagerType {
    Yarn,
    Npm,
}

struct Packager {
    type_: PackagerType,
    source: Source,
    version: Version,
}

#[derive(Default)]
struct Toolchain {
    node_runtimes: Vec<Node>,
    packagers: Vec<Packager>,
    packages: Vec<Package>,
    tools: Vec<Tool>,
}

#[derive(StructOpt)]
pub(crate) struct List {
    /// Display
    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,

    /// Specify the output format.
    ///
    /// Defaults to `human` for TTYs, `plain` otherwise.
    #[structopt(long = "format", raw(possible_values = r#"&["human", "plain"]"#))]
    format: Option<Format>,

    /// Show the currently-active tool(s).
    ///
    /// Equivalent to `volta list` when not specifying a specific tool.
    #[structopt(long = "current")]
    current: bool,

    /// Show your default tool(s).
    #[structopt(long = "default")]
    default: bool,
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
    type Err = (); // Use Never/`!` when it stabilizes
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
    fn output_format(&self) -> Format {
        // We start by checking if the user has explicitly set a value: if they
        // have, that trumps our TTY-checking. Then, if the user has *not*
        // specified an option, we use `Human` mode for TTYs and `Plain` for
        // non-TTY contexts.
        self.format.unwrap_or(if atty::is(atty::Stream::Stdout) {
            Format::Human
        } else {
            Format::Plain
        })
    }
}

impl Command for List {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::List);

        let inventory = session.inventory()?;
        let project = session.project();
        let formatter = match self.output_format() {
            Format::Human => human::format,
            Format::Plain => plain::format,
        };

        let toolchain_to_display = match self.subcommand {
            Some(Subcommand::All) => (),
            Some(Subcommand::Node) => (),
            Some(Subcommand::Yarn) => (),
            Some(Subcommand::PackageOrTool { name }) => (),
            None => (),
        };

        session.add_event_end(ActivityKind::List, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
