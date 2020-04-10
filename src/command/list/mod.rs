mod human;
mod plain;
mod toolchain;

use std::{fmt, path::PathBuf, str::FromStr};

use semver::Version;
use structopt::StructOpt;

use crate::command::Command;
use toolchain::Toolchain;
use volta_core::inventory::package_configs;
use volta_core::project::Project;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::PackageConfig;
use volta_fail::{ExitCode, Fallible};

#[derive(Copy, Clone, PartialEq)]
enum Format {
    Human,
    Plain,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Format::Human),
            "plain" => Ok(Format::Plain),
            _ => Err("No".into()),
        }
    }
}

/// The source of a given item, from the perspective of a user.
///
/// Note: this is distinct from `volta_core::platform::sourced::Source`, which
/// represents the source only of a `Platform`, which is a composite structure.
/// By contrast, this `Source` is concerned *only* with a single item.
#[derive(Clone, PartialEq, Debug)]
enum Source {
    /// The item is from a project. The wrapped `PathBuf` is the path to the
    /// project's `package.json`.
    Project(PathBuf),

    /// The item is the user's default.
    Default,

    /// The item is one that has been *fetched* but is not *installed* anywhere.
    None,
}

impl Source {
    fn allowed_with(&self, filter: &Filter) -> bool {
        match filter {
            Filter::Default => self == &Source::Default,
            Filter::Current => match self {
                Source::Default | Source::Project(_) => true,
                _ => false,
            },
            Filter::None => true,
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Source::Project(path) => format!(" (current @ {})", path.display()),
                Source::Default => String::from(" (default)"),
                Source::None => String::from(""),
            }
        )
    }
}

/// A package and its associated tools, for displaying to the user as part of
/// their toolchain.
struct PackageDetails {
    /// The name of the package.
    pub name: String,
    /// The package's own version.
    pub version: Version,
}

enum Package {
    Default {
        details: PackageDetails,
        /// The version of Node the package is installed against.
        node: Version,
        /// The names of the tools associated with the package.
        tools: Vec<String>,
    },
    Project {
        details: PackageDetails,
        /// The version of Node the package is installed against.
        node: Version,
        /// The names of the tools associated with the package.
        tools: Vec<String>,
        path: PathBuf,
    },
    Fetched(PackageDetails),
}

impl Package {
    fn new(config: &PackageConfig, source: &Source) -> Package {
        let details = PackageDetails {
            name: config.name.clone(),
            version: config.version.clone(),
        };

        match source {
            Source::Default => Package::Default {
                details,
                node: config.platform.node.clone(),
                tools: config.bins.clone(),
            },
            Source::Project(path) => Package::Project {
                details,
                node: config.platform.node.clone(),
                tools: config.bins.clone(),
                path: path.clone(),
            },
            Source::None => Package::Fetched(details),
        }
    }

    fn from_inventory_and_project(project: Option<&Project>) -> Fallible<Vec<Package>> {
        package_configs().map(|configs| {
            configs
                .iter()
                .map(|config| {
                    let source = Self::source(&config.name, &config.version, project);
                    Package::new(&config, &source)
                })
                .collect()
        })
    }

    fn source(name: &str, version: &Version, project: Option<&Project>) -> Source {
        match project {
            Some(project) if project.has_dependency(name, version) => {
                Source::Project(project.package_file())
            }
            _ => Source::Default,
        }
    }
}

#[derive(Clone)]
struct Node {
    pub source: Source,
    pub version: Version,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PackageManagerKind {
    Npm,
    Yarn,
}

impl fmt::Display for PackageManagerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PackageManagerKind::Npm => "npm",
                PackageManagerKind::Yarn => "yarn",
            }
        )
    }
}

#[derive(Clone)]
struct PackageManager {
    kind: PackageManagerKind,
    source: Source,
    version: Version,
}

/// How (if at all) should the list query be narrowed?
enum Filter {
    /// Display only the currently active tool(s).
    ///
    /// For example, if the user queries `volta list --current yarn`, show only
    /// the version of Yarn currently in use: project, default, or none.
    Current,

    /// Show only the user's default tool(s).
    ///
    /// For example, if the user queries `volta list --default node`, show only
    /// the user's default Node version.
    Default,

    /// Do not filter at all. Show all tool(s) matching the query.
    None,
}

#[derive(StructOpt)]
pub(crate) struct List {
    // Note: we implement the subcommand as an `Option<String>` instead of an
    // `Option<Subcommand>` with `impl FromStr for Subcommand` for `StructOpt`
    // because StructOpt does not currently support custom parsing for enum
    // variants (as detailed in commit 5f9214ae).
    /// The tool to lookup: `all`, `node`, `yarn`, or the name of a package or binary.
    #[structopt(name = "tool")]
    subcommand: Option<String>,

    /// Specify the output format.
    ///
    /// Defaults to `human` for TTYs, `plain` otherwise.
    #[structopt(long = "format", raw(possible_values = r#"&["human", "plain"]"#))]
    format: Option<Format>,

    /// Show the currently-active tool(s).
    ///
    /// Equivalent to `volta list` when not specifying a specific tool.
    #[structopt(long = "current", short = "c", conflicts_with = "default")]
    current: bool,

    /// Show your default tool(s).
    #[structopt(long = "default", short = "d", conflicts_with = "current")]
    default: bool,
}

/// Which tool should we look up?
enum Subcommand {
    /// Show every item in the toolchain.
    All,

    /// Show locally cached Node versions.
    Node,

    /// Show locally cached npm versions.
    Npm,

    /// Show locally cached Yarn versions.
    Yarn,

    /// Show locally cached versions of a package or a package binary.
    PackageOrTool { name: String },
}

impl From<&str> for Subcommand {
    fn from(s: &str) -> Self {
        match s {
            "all" => Subcommand::All,
            "node" => Subcommand::Node,
            "npm" => Subcommand::Npm,
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

    fn subcommand(&self) -> Option<Subcommand> {
        self.subcommand.as_ref().map(|s| s.as_str().into())
    }
}

impl Command for List {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::List);

        let project = session.project()?;
        let default_platform = session.default_platform()?;
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

        let toolchain = match self.subcommand() {
            // For no subcommand, show the user's current toolchain
            None => Toolchain::active(project, default_platform)?,
            Some(Subcommand::All) => Toolchain::all(project, default_platform)?,
            Some(Subcommand::Node) => Toolchain::node(project, default_platform, &filter)?,
            Some(Subcommand::Npm) => Toolchain::npm(project, default_platform, &filter)?,
            Some(Subcommand::Yarn) => Toolchain::yarn(project, default_platform, &filter)?,
            Some(Subcommand::PackageOrTool { name }) => {
                Toolchain::package_or_tool(&name, project, &filter)?
            }
        };

        if let Some(string) = format(&toolchain) {
            println!("{}", string)
        };

        session.add_event_end(ActivityKind::List, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
