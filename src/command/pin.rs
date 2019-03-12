use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::style::{display_error, ErrorContext};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Pin {
    /// The tool to install, e.g. `node` or `npm` or `yarn`
    tool: String,

    /// The version of the tool to install, e.g. `1.2.3` or `latest`
    version: String,
}

impl Command for Pin {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Pin);

        let version = VersionSpec::parse(&self.version)?;
        let tool = ToolSpec::from_str_and_version(&self.tool, version);
        session.pin(&tool)?;

        if let Some(project) = session.project()? {
            let errors = project.autoshim();

            for error in errors {
                display_error(ErrorContext::Notion, &error);
            }
        }

        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(())
    }
}
