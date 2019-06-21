mod human;
mod plain;

use std::fmt;
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

#[derive(Clone)]
enum Source {
    Project(PathBuf),
    User,
    None,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Source::Project(path) => format!(" (current @ {})", path.display()),
                Source::User => String::from(" (default)"),
                Source::None => String::from(""),
            }
        )
    }
}

struct Package {
    /// The name of the package.
    pub name: String,
    /// Where the package is specified.
    pub source: Source,
    /// The package's own version.
    pub version: Version,
    /// The version of Node the package is installed against.
    pub node: Version,
    /// The names of the tools associated with the package.
    pub tools: Vec<String>,
}

#[derive(Clone)]
struct Node {
    pub source: Source,
    pub version: Version,
}

#[derive(Clone)]
enum PackageManagerType {
    Yarn,
    Npm,
}

impl fmt::Display for PackageManagerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PackageManagerType::Npm => "npm",
                PackageManagerType::Yarn => "yarn",
            }
        )
    }
}

#[derive(Clone)]
struct PackageManager {
    type_: PackageManagerType,
    source: Source,
    version: Version,
}

enum Toolchain {
    Node(Vec<Node>),
    PackageManagers(Vec<PackageManager>),
    Packages(Vec<Package>),
    Tool {
        name: String,
        host_packages: Vec<Package>,
    },
    Active {
        runtime: Option<Node>,
        package_manager: Option<PackageManager>,
        packages: Vec<Package>,
    },
    All {
        runtimes: Vec<Node>,
        package_managers: Vec<PackageManager>,
        packages: Vec<Package>,
    },
}

impl Toolchain {
    fn active(filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }

    fn all() -> Fallible<Toolchain> {
        unimplemented!()
    }

    fn node(filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }

    fn yarn(filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }

    fn package_or_tool(name: &str, filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }
}

enum Filter {
    Current,
    Default,
    None,
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
    #[structopt(long = "current", conflicts_with = "default")]
    current: bool,

    /// Show your default tool(s).
    #[structopt(long = "default", conflicts_with = "current")]
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

impl From<&str> for Subcommand {
    fn from(s: &str) -> Self {
        match s {
            "all" => Subcommand::All,
            "node" => Subcommand::Node,
            "yarn" => Subcommand::Yarn,
            s => Subcommand::PackageOrTool { name: s.into() },
        }
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
        let format = match self.output_format() {
            Format::Human => human::format,
            Format::Plain => plain::format,
        };

        let filter = match (self.current, self.default) {
            (true, false) => Filter::Current,
            (false, true) => Filter::Default,
            (true, true) => unreachable!("simultaneous `current` and `default` forbidden by clap"),
            _ => Filter::None,
        };

        let toolchain_to_display: Toolchain = match self.subcommand {
            // For no subcommand, show the user's current toolchain
            None => Toolchain::active(&filter)?,
            Some(Subcommand::All) => Toolchain::all()?,
            Some(Subcommand::Node) => Toolchain::node(&filter)?,
            Some(Subcommand::Yarn) => Toolchain::yarn(&filter)?,
            Some(Subcommand::PackageOrTool { name }) => Toolchain::package_or_tool(&name, &filter)?,
        };

        if let Some(string) = format(&toolchain_to_display) {
            println!("{}", string);
        };

        session.add_event_end(ActivityKind::List, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use semver::Version;

    lazy_static! {
        static ref NODE_PROJECT_VERSION: Version = Version::from((12, 2, 0));
        static ref NODE_DEFAULT_VERSION: Version = Version::from((10, 15, 3));
        static ref NODE_VERSION_11: Version = Version::from((11, 9, 0));
        static ref NODE_VERSION_8: Version = Version::from((8, 16, 0));

        static ref YARN_DEFAULT: Version = Version::from((1, 16, 0));
        static ref YARN_PROJECT: Version = Version::from((1, 12, 3));

        static ref PROJECT_PATH: PathBuf = PathBuf::from("~/node-and-yarn/project.json");

        static ref CREATE_REACT_APP: Version = Version::from((3, 0, 1));

        static ref EMBER_DEFAULT: Version = Version::from((3, 10, 0));
        static ref EMBER_LTS: Version = Version::from((3, 8, 2));

        static ref TS_RECENT_VERSION: Version = Version::from((3, 4, 3));
        static ref TS_DEFAULT_VERSION: Version = Version::from((3, 4, 1));

        static ref YARN_DEDUPLICATE: Version = Version::from((1, 1, 1));
    }

    mod bare {
        #[test]
        fn human() {
            unimplemented!();
        }

        #[test]
        fn plain() {
            unimplemented!();
        }
    }

    mod all {
        #[test]
        fn human() {
            unimplemented!();
        }

        #[test]
        fn plain() {
            unimplemented!();
        }
    }

    mod package {
        #[test]
        fn human() {
            unimplemented!();
        }

        #[test]
        fn plain() {
            unimplemented!();
        }
    }
}