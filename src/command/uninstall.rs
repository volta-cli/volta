use volta_core::error::{ExitCode, Fallible};
use volta_core::session::{ActivityKind, Session};
use volta_core::tool;

use crate::command::Command;

#[derive(clap::Args)]
pub(crate) struct Uninstall {
    /// The tool to uninstall, like `ember-cli-update`, `typescript`, or <package>
    tool: String,
}

impl Command for Uninstall {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Uninstall);

        let tool = tool::Spec::try_from_str(&self.tool)?;
        tool.resolve(session)?.uninstall(session)?;

        session.add_event_end(ActivityKind::Uninstall, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
