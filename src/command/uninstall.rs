use structopt::StructOpt;

use jetson_core::session::{ActivityKind, Session};
use jetson_core::tool::ToolSpec;
use jetson_core::version::VersionSpec;
use jetson_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Uninstall {
    /// The tool to uninstall, e.g. `node`, `npm`, `yarn`, or <package>
    tool: String,
}

impl Command for Uninstall {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Uninstall);

        let version = VersionSpec::default();
        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        tool.uninstall(session)?;

        session.add_event_end(ActivityKind::Uninstall, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
