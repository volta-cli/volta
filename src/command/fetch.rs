use structopt::StructOpt;

use jetson_core::session::{ActivityKind, Session};
use jetson_core::tool::ToolSpec;
use jetson_core::version::VersionSpec;
use jetson_fail::{ExitCode, Fallible};

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

        match tool {
            ToolSpec::Node(version) => {
                session.fetch_node(&version)?;
            }
            ToolSpec::Yarn(version) => {
                session.fetch_yarn(&version)?;
            }
            ToolSpec::Npm(_version) => {
                // ISSUE(#292): Implement install for npm
                unimplemented!("Fetching npm is not supported yet");
            }
            ToolSpec::Package(name, version) => {
                session.fetch_package(&name, &version)?;
            }
        }
        session.add_event_end(ActivityKind::Fetch, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
