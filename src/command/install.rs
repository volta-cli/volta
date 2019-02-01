use structopt::StructOpt;

use notion_core::session::{ActivityKind, Session};
use notion_core::tool::ToolSpec;
use notion_core::version::VersionSpec;
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Install {
    /// The tool to install, e.g. `node` or `npm` or `yarn`
    tool: String,

    /// The version of the tool to install, e.g. `1.2.3` or `latest`
    version: Option<String>,
}

impl Command for Install {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Install);
        match self {
            Install::Help => {
                Help::Command(CommandName::Install).run(session)?;
            }
            Install::Tool(toolspec) => {
                match toolspec {
                    ToolSpec::Node(version) => session.install_node(&version)?,
                    ToolSpec::Yarn(version) => session.install_yarn(&version)?,
                    ToolSpec::Npm(version) => unimplemented!("TODO"), // session.install_npm(&version)?,
                    ToolSpec::Package(name, version) => session.install_package(&name, &version)?,
                }
            }
        };

        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        session.install(&tool)?;

        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
