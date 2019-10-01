use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::platform::Source;
use crate::session::{ActivityKind, Session};
use crate::style::tool_version;

use log::debug;
use volta_fail::Fallible;

pub(crate) fn command<A>(
    args: A,
    session: &mut Session,
    current_dir: Option<&Path>,
) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Node);

    match session.current_platform()? {
        Some(platform) => {
            let source = match platform.source() {
                Source::Project | Source::ProjectNodeDefaultYarn => "project",
                Source::Default => "default",
            };
            let version = tool_version("node", platform.node());
            debug!("Using {} from {} configuration", version, source);

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(
                OsStr::new("node"),
                args,
                &path,
                current_dir,
            ))
        }
        None => {
            debug!("Could not find Volta-managed node, delegating to system");
            ToolCommand::passthrough(
                OsStr::new("node"),
                args,
                ErrorDetails::NoPlatform,
                current_dir,
            )
        }
    }
}
