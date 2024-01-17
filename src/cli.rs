use clap::Parser;

use crate::command::{self, Command};
use volta_core::error::{ExitCode, Fallible};
use volta_core::session::Session;

#[derive(Parser)]
#[command(
    name = "Volta",
    about = "The JavaScript Launcher ⚡",
    long_about = "The JavaScript Launcher ⚡

    To install a tool in your toolchain, use `volta install`.
    To pin your project's runtime or package manager, use `volta pin`.",
    color = clap::ColorChoice::Auto,
    disable_version_flag = true,
)]
pub(crate) struct Volta {
    #[command(subcommand)]
    pub(crate) command: Option<Subcommand>,

    #[arg(long = "verbose", help = "Enables verbose diagnostics", global = true)]
    pub(crate) verbose: bool,

    #[arg(
        long = "quiet",
        help = "Prevents unnecessary output",
        global = true,
        conflicts_with = "verbose",
        aliases = &["silent"]
    )]
    pub(crate) quiet: bool,

    #[arg(
        short = 'v',
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
            Volta::parse_from(["volta", "help"].iter()).run(session)
        }
    }
}

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Fetches a tool to the local machine
    #[command(name = "fetch")]
    Fetch(command::Fetch),

    /// Installs a tool in your toolchain
    #[command(name = "install")]
    Install(command::Install),

    /// Uninstalls a tool from your toolchain
    #[command(name = "uninstall")]
    Uninstall(command::Uninstall),

    /// Pins your project's runtime or package manager
    #[command(name = "pin")]
    Pin(command::Pin),

    /// Displays the current toolchain
    #[command(name = "list", alias = "ls")]
    List(command::List),

    /// Generates Volta completions
    #[command(
        name = "completions",
        arg_required_else_help = true,
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
    #[command(name = "which")]
    Which(command::Which),

    #[command(
        name = "use",
        long_about = crate::command::r#use::USAGE,
        hide = true,
    )]
    Use(command::Use),

    /// Enables Volta for the current user / shell
    #[command(name = "setup")]
    Setup(command::Setup),

    /// Run a command with custom Node, npm, pnpm, and/or Yarn versions
    #[command(name = "run")]
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
