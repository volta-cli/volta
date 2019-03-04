use serde::Deserialize;

use notion_core::error::ErrorDetails;
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
    version: Option<String>,
}

impl Command for Pin {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Pin);

        let version = match self.version {
            Some(version_string) => VersionSpec::parse(&version_string)?,
            None => VersionSpec::default(),
        };

        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        match toolspec {
            ToolSpec::Node(version) => session.pin_node(&version)?,
            ToolSpec::Yarn(version) => session.pin_yarn(&version)?,
            ToolSpec::Npm(version) => session.pin_npm(&version)?,
            ToolSpec::Package(_name, _version) => throw!(ErrorDetails::CannotPinPackage),
        }

        if let Some(project) = session.project()? {
            let errors = project.autoshim();

            for error in errors {
                display_error(ErrorContext::Notion, &error);
            }
        }

        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
