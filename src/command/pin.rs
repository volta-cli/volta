use structopt::StructOpt;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Pin {
    /// The tool to pin, e.g. `node` or `npm` or `yarn`, with optional version.
    #[structopt(multiple = true, parse(try_from_str = "ToolSpec::try_from_str"))]
    tools: Vec<ToolSpec>,
}

impl Command for Pin {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        for tool in self.tools {
            session.add_event_start(ActivityKind::Pin);
            match tool {
                ToolSpec::Node(version) => session.pin_node(&version)?,
                ToolSpec::Yarn(version) => session.pin_yarn(&version)?,
                // ISSUE(#292): Implement install for npm
                ToolSpec::Npm(_version) => unimplemented!("Pinning npm is not supported yet"),
                ToolSpec::Package(name, _version) => {
                    throw!(ErrorDetails::CannotPinPackage { package: name })
                }
            }
            session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        }

        Ok(ExitCode::Success)
    }
}
