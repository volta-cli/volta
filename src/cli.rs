use structopt::StructOpt;

use notion_core::session::Session;
use notion_fail::Fallible;

use crate::command::{self, Command};

#[derive(StructOpt)]
#[structopt(
    name = "Notion",
    about = "The hassle-free JavaScript toolchain manager",
    author = "",
    usage = "notion [flags] [<command> <args> ...]

    To install a tool in your user toolchain, use `notion install`.
    To pin a tool in a project toolchain, use `notion pin`.",
    raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp"),
    raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(global_setting = "structopt::clap::AppSettings::ColorAlways"),
    raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"),
    raw(global_setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage")
)]
pub(crate) struct Notion {
    #[structopt(subcommand)]
    pub(crate) command: Subcommand,

    // not yet implemented!
    #[structopt(long = "verbose", help = "switch on verbosity", global = true)]
    #[allow(dead_code)]
    pub(crate) verbose: bool,
}

#[derive(StructOpt)]
pub(crate) enum Subcommand {
    /// Fetch a tool to the local machine
    #[structopt(name = "fetch", author = "")]
    Fetch(command::Fetch),

    /// Install a tool in the user toolchain.
    #[structopt(name = "install", author = "")]
    Install(command::Install),

    /// Select a tool for the current project's toolchain
    #[structopt(name = "pin", author = "")]
    Pin(command::Pin),

    /// Get or set configuration values
    #[structopt(name = "config", author = "")]
    Config(command::Config),

    /// Display the currently activated Node version
    #[structopt(name = "current", author = "")]
    Current(command::Current),

    /// Disable Notion in the current shell
    #[structopt(name = "deactivate", author = "")]
    Deactivate(command::Deactivate),

    /// Re-enable Notion in the current shell
    #[structopt(name = "activate", author = "")]
    Activate(command::Activate),

    /// Generate Notion completions.
    #[structopt(
        name = "completions",
        author = "",
        long_about = "Generate Notion completions.

By default, completions will be generated for the value of your current shell,
shell, i.e. the value of `SHELL`. If you set the `<shell>` option, completions
will be generated for that shell instead.

If you specify a directory, the completions will be written to a file there;
otherwise, they will be written to `stdout`.
    "
    )]
    Completions(command::Completions),
}

impl Subcommand {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<()> {
        match self {
            Subcommand::Fetch(fetch) => fetch.run(session),
            Subcommand::Install(install) => install.run(session),
            Subcommand::Pin(pin) => pin.run(session),
            Subcommand::Config(config) => config.run(session),
            Subcommand::Current(current) => current.run(session),
            Subcommand::Deactivate(deactivate) => deactivate.run(session),
            Subcommand::Activate(activate) => activate.run(session),
            Subcommand::Completions(completions) => completions.run(session),
        }
    }
}
