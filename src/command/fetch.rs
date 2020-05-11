use structopt::StructOpt;

use volta_core::error::{ExitCode, Fallible};
use volta_core::session::{ActivityKind, Session};
use volta_core::tool;

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Fetch {
    /// Tools to fetch, like `node`, `yarn@latest` or `your-package@^14.4.3`.
    #[structopt(name = "tool[@version]", required = true, min_values = 1)]
    tools: Vec<String>,
}

impl Command for Fetch {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Fetch);

        for tool in tool::Spec::from_strings(&self.tools, "fetch")? {
            tool.resolve(session)?.fetch(session)?;
        }

        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
