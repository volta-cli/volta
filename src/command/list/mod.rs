mod human;
mod plain;
mod toolchain;

use std::io::IsTerminal as _;
use std::{fmt, path::PathBuf, str::FromStr};

use node_semver::Version;

use crate::command::Command;
use toolchain::Toolchain;
use volta_core::error::{ExitCode, Fallible};
use volta_core::inventory::package_configs;
use volta_core::project::Project;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::PackageConfig;

#[derive(clap::ValueEnum, Copy, Clone)]
enum Format {
    Human,
    Plain,
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
            Filter::Current => matches!(self, Source::Default | Source::Project(_)),
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
        name: String,
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
                name: details.name,
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
                    let source = Self::source(&config.name, project);
                    Package::new(config, &source)
                })
                .collect()
        })
    }

    fn source(name: &str, project: Option<&Project>) -> Source {
        match project {
            Some(project) if project.has_direct_dependency(name) => {
                Source::Project(project.manifest_file().to_owned())
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
    Pnpm,
    Yarn,
}

impl fmt::Display for PackageManagerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PackageManagerKind::Npm => "npm",
                PackageManagerKind::Pnpm => "pnpm",
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

#[derive(clap::Args)]
pub(crate) struct List {
    /// The tool to lookup - `all`, `node`, `npm`, `yarn`, `pnpm`, or the name
    /// of a package or binary.
    #[arg(value_name = "tool")]
    subcommand: Option<Subcommand>,

    /// Specify the output format.
    ///
    /// Defaults to `human` for TTYs, `plain` otherwise.
    #[arg(long)]
    format: Option<Format>,

    /// Show the currently-active tool(s).
    ///
    /// Equivalent to `volta list` when not specifying a specific tool.
    #[arg(short, long, conflicts_with = "default")]
    current: bool,

    /// Show your default tool(s).
    #[arg(short, long, conflicts_with = "current")]
    default: bool,
}

/// Which tool should we look up?
#[derive(Clone)]
enum Subcommand {
    /// Show every item in the toolchain.
    All,

    /// Show locally cached Node versions.
    Node,

    /// Show locally cached npm versions.
    Npm,

    /// Show locally cached pnpm versions.
    Pnpm,

    /// Show locally cached Yarn versions.
    Yarn,

    /// Show locally cached versions of a package or a package binary.
    PackageOrTool { name: String },
}

impl FromStr for Subcommand {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "all" => Subcommand::All,
            "node" => Subcommand::Node,
            "npm" => Subcommand::Npm,
            "pnpm" => Subcommand::Pnpm,
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
        self.format.unwrap_or(if std::io::stdout().is_terminal() {
            Format::Human
        } else {
            Format::Plain
        })
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

        let toolchain = match self.subcommand {
            // For no subcommand, show the user's current toolchain
            None => Toolchain::active(project, default_platform)?,
            Some(Subcommand::All) => Toolchain::all(project, default_platform)?,
            Some(Subcommand::Node) => Toolchain::node(project, default_platform, &filter)?,
            Some(Subcommand::Npm) => Toolchain::npm(project, default_platform, &filter)?,
            Some(Subcommand::Pnpm) => Toolchain::pnpm(project, default_platform, &filter)?,
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
