use notion_core::session::Session;
use notion_fail::{Fallible, ResultExt};
use semver::Version;

use ::Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: String
}

pub(crate) enum Uninstall {
    Help,
    Default(Version)
}

impl Command for Uninstall {
    type Args = Args;

    const USAGE: &'static str = "
Uninstall a toolchain from the local machine

Usage:
    notion uninstall <version>
    notion uninstall -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self { Uninstall::Help }

    fn parse(_: Notion, Args { arg_version }: Args) -> Fallible<Self> {
        let version = Version::parse(&arg_version).unknown()?;
        Ok(Uninstall::Default(version))
    }

    fn run(self) -> Fallible<bool> {
        match self {
            Uninstall::Help => {
                Help::Command(CommandName::Uninstall).run()
            }
            Uninstall::Default(version) => {
                let mut session = Session::new()?;
                session.catalog_mut()?.uninstall_node(&version)?;
                Ok(true)
            }
        }
    }

}
