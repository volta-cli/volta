use structopt::StructOpt;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Fetch {
    /// Tools to fetch, like `node`, `yarn@latest` or `your-package@^14.4.3`.
    #[structopt(name = "tool[@version]", required = true, min_values = 1)]
    tools: Vec<String>,
}

impl Command for Fetch {
    fn run(mut self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Fetch);

        for tool in ToolSpec::from_strings(&mut self.tools, "fetch")? {
            match tool {
                ToolSpec::Node(version) => {
                    session.fetch_node(&version)?;
                }
                ToolSpec::Yarn(version) => {
                    session.fetch_yarn(&version)?;
                }
                ToolSpec::Npm(_version) => {
                    // ISSUE(#292): Implement install for npm
                    throw!(ErrorDetails::Unimplemented {
                        feature: "Fetching npm".into()
                    });
                }
                ToolSpec::Package(name, version) => {
                    session.fetch_package(&name, &version)?;
                }
            }
        }

        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
