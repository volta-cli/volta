// TODO: all of this haha
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
    // TODO: need this?
    // /// The version of the tool to uninstall, e.g. `1.2.3` or `latest`
    // version: Option<String>,
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
