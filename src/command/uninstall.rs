use volta_core::error::{ErrorKind, ExitCode, Fallible};
use volta_core::session::{ActivityKind, Session};
use volta_core::tool;
use volta_core::version::VersionSpec;

use crate::command::Command;

#[derive(clap::Args)]
pub(crate) struct Uninstall {
    /// The tool to uninstall, like `ember-cli-update`, `typescript`, or <package>
    tool: String,
}

impl Command for Uninstall {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Uninstall);

        let tool = tool::Spec::try_from_str(&self.tool)?;

        // For packages, specifically report that we do not support uninstalling
        // specific versions. For runtimes and package managers, we currently
        // *intentionally* let this fall through to inform the user that we do
        // not support uninstalling those *at all*.
        if let tool::Spec::Package(_name, version) = &tool {
            let VersionSpec::None = version else {
                return Err(ErrorKind::Unimplemented {
                    feature: "uninstalling specific versions of tools".into(),
                }
                .into());
            };
        }

        tool.uninstall()?;

        session.add_event_end(ActivityKind::Uninstall, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
