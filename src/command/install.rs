use structopt::StructOpt;

use volta_core::session::{ActivityKind, Session};
use volta_core::tool::Spec;
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Install {
    /// Tools to install, like `node`, `yarn@latest` or `your-package@^14.4.3`.
    #[structopt(name = "tool[@version]", required = true, min_values = 1)]
    tools: Vec<String>,
}

impl Command for Install {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Install);

        for tool in Spec::from_strings(&self.tools, "install")? {
            tool.resolve(session)?.install(session)?;
        }

        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
