use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Install {
    /// The tool to install, e.g. `node` or `npm` or `yarn`, with optional version.
    #[structopt(multiple = true, parse(try_from_str = "ToolSpec::try_from_str"))]
    tools: Vec<ToolSpec>,
}

impl Command for Install {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        for tool in self.tools {
            session.add_event_start(ActivityKind::Install);
            tool.install(session)?;
            session.add_event_end(ActivityKind::Install, ExitCode::Success);
        }

        Ok(ExitCode::Success)
    }
}
