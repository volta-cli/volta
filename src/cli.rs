use structopt::StructOpt;

use crate::command::{self, Command};
use volta_core::session::Session;
use volta_fail::{ExitCode, Fallible};

#[derive(StructOpt)]
#[structopt(
    name = "Volta",
    about = "The JavaScript Launcher ⚡",
    author = "",
    long_about = "The JavaScript Launcher ⚡

    To install a tool in your toolchain, use `volta install`.
    To pin your project's runtime or package manager, use `volta pin`.",
    raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(global_setting = "structopt::clap::AppSettings::ColorAuto"),
    raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"),
    raw(global_setting = "structopt::clap::AppSettings::DisableVersion"),
    raw(global_setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage"),
    raw(global_setting = "structopt::clap::AppSettings::VersionlessSubcommands")
)]
pub(crate) struct Volta {
    #[structopt(subcommand)]
    pub(crate) command: Option<Subcommand>,

    #[structopt(long = "verbose", help = "Enables verbose diagnostics", global = true)]
    pub(crate) verbose: bool,

    #[structopt(
        long = "quiet",
        help = "Prevents unnecessary output",
        global = true,
        conflicts_with = "verbose",
        raw(aliases = r#"&["silent"]"#)
    )]
    pub(crate) quiet: bool,

    #[structopt(
        short = "v",
        long = "version",
        help = "Prints the current version of Volta"
    )]
    pub(crate) version: bool,
}

impl Volta {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        if self.version {
            // suffix indicator for dev build
            if cfg!(debug_assertions) {
                println!("{}-dev", env!("CARGO_PKG_VERSION"));
            } else {
                println!("{}", env!("CARGO_PKG_VERSION"));
            }
            Ok(ExitCode::Success)
        } else if let Some(command) = self.command {
            command.run(session)
        } else {
            Volta::from_iter(["volta", "help"].iter()).run(session)
        }
    }
}

#[derive(StructOpt)]
pub(crate) enum Subcommand {
    /// Fetches a tool to the local machine
    #[structopt(name = "fetch", author = "", version = "")]
    Fetch(command::Fetch),

    /// Installs a tool in your toolchain
    #[structopt(name = "install", author = "", version = "")]
    Install(command::Install),

    /// Uninstalls a tool from your toolchain
    #[structopt(name = "uninstall", author = "", version = "")]
    Uninstall(command::Uninstall),

    /// Pins your project's runtime or package manager
    #[structopt(name = "pin", author = "", version = "")]
    Pin(command::Pin),

    /// Displays the current toolchain
    #[structopt(name = "list", alias = "ls", author = "", version = "")]
    List(command::List),

    /// Generates Volta completions
    #[structopt(
        name = "completions",
        author = "",
        version = "",
        raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp"),
        long_about = "Generates Volta completions

By default, completions will be generated for the value of your current shell,
shell, i.e. the value of `SHELL`. If you set the `<shell>` option, completions
will be generated for that shell instead.

If you specify a directory, the completions will be written to a file there;
otherwise, they will be written to `stdout`.
    "
    )]
    Completions(command::Completions),

    /// Locates the actual binary that will be called by Volta
    #[structopt(name = "which", author = "", version = "")]
    Which(command::Which),

    #[structopt(
        name = "use",
        author = "",
        version = "",
        template = "{usage}",
        raw(
            usage = "crate::command::r#use::USAGE",
            setting = "structopt::clap::AppSettings::Hidden"
        )
    )]
    Use(command::Use),

    /// Enables Volta for the current user / shell
    #[structopt(name = "setup", author = "", version = "")]
    Setup(command::Setup),

    /// Run a command with custom Node, npm, and/or Yarn versions
    #[structopt(name = "run", author = "", version = "")]
    #[structopt(raw(setting = "structopt::clap::AppSettings::AllowLeadingHyphen"))]
    #[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
    Run(command::Run),
}

impl Subcommand {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        match self {
            Subcommand::Fetch(fetch) => fetch.run(session),
            Subcommand::Install(install) => install.run(session),
            Subcommand::Uninstall(uninstall) => uninstall.run(session),
            Subcommand::Pin(pin) => pin.run(session),
            Subcommand::List(list) => list.run(session),
            Subcommand::Completions(completions) => completions.run(session),
            Subcommand::Which(which) => which.run(session),
            Subcommand::Use(r#use) => r#use.run(session),
            Subcommand::Setup(setup) => setup.run(session),
            Subcommand::Run(run) => run.run(session),
        }
    }
}
