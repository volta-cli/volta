use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Fetch {
    /// The tool to install, e.g. `node` or `npm` or `yarn`
    tool: String,

    /// The version of the tool to install, e.g. `1.2.3` or `latest`
    version: String,
}

impl Command for Fetch {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Fetch);

        let version = VersionSpec::parse(&self.version)?;
        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        session.fetch(&tool)?;

        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
