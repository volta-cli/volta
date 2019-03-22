use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Uninstall {
    /// The tool to uninstall, e.g. `node`, `npm`, `yarn`, or <package>
    tool: String,

    /// Remove the cached tarball files
    #[structopt(short = "f", long = "full")]
    remove_all: bool,
}

impl Command for Uninstall {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Uninstall);

        let version = VersionSpec::default();
        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        tool.uninstall(session, self.remove_all)?;

        session.add_event_end(ActivityKind::Uninstall, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
