use notion_core::session::{ActivityKind, Session};
use notion_fail::{Fallible, NotionFail, ResultExt};
use semver::{SemVerError, Version};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Fail, Debug)]
#[fail(display = "Could not parse version: {}", error)]
pub(crate) struct VersionParseError {
    pub(crate) error: SemVerError,
}

impl VersionParseError {
    pub(crate) fn from_semver_err(error: &SemVerError) -> Self {
        VersionParseError {
            error: error.clone(),
        }
    }
}

impl NotionFail for VersionParseError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> i32 {
        4
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    arg_version: String,
}

pub(crate) enum Uninstall {
    Help,
    Default(Version),
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

    fn help() -> Self {
        Uninstall::Help
    }

    fn parse(_: Notion, Args { arg_version }: Args) -> Fallible<Self> {
        let version =
            Version::parse(&arg_version).with_context(VersionParseError::from_semver_err)?;
        Ok(Uninstall::Default(version))
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Uninstall);
        let result = match self {
            Uninstall::Help => Help::Command(CommandName::Uninstall).run(session),
            Uninstall::Default(version) => {
                session.catalog_mut()?.uninstall_node(&version)?;
                Ok(true)
            }
        };
        session.add_event_end(ActivityKind::Uninstall, 0);
        result
    }
}
