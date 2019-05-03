use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Install {
    /// Tools to install, like `node`, `yarn@latest` or `your-package@^14.4.3`.
    #[structopt(name = "tool[@version]", required = true, min_values = 1)]
    tools: Vec<String>,
}

impl Command for Install {
    fn run(mut self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Install);

        for tool in ToolSpec::from_strings(&mut self.tools, "install")? {
            tool.install(session)?;
        }

        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
