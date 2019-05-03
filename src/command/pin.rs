use structopt::StructOpt;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Pin {
    /// Tools to pin, like `node@lts` or `yarn@^1.14`.
    #[structopt(name = "tool[@version]", required = true, min_values = 1)]
    tools: Vec<String>,
}

impl Command for Pin {
    fn run(mut self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Pin);

        for tool in ToolSpec::from_strings(&mut self.tools, "pin")? {
            match tool {
                ToolSpec::Node(version) => session.pin_node(&version)?,
                ToolSpec::Yarn(version) => session.pin_yarn(&version)?,
                // ISSUE(#292): Implement install for npm
                ToolSpec::Npm(_version) => throw!(ErrorDetails::Unimplemented {
                    feature: "Pinning npm".into()
                }),
                ToolSpec::Package(name, _version) => {
                    throw!(ErrorDetails::CannotPinPackage { package: name })
                }
            }
        }

        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
