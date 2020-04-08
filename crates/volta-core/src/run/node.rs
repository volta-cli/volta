use std::ffi::OsStr;

use super::{debug_tool_message, ToolCommand};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use log::debug;
use volta_fail::Fallible;

pub(crate) fn command(session: &mut Session) -> Fallible<ToolCommand> {
    session.add_event_start(ActivityKind::Node);

    match session.current_platform()? {
        Some(platform) => {
            debug_tool_message("node", &platform.node());

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("node"), &path))
        }
        None => {
            debug!("Could not find Volta-managed node, delegating to system");
            ToolCommand::passthrough(OsStr::new("node"), ErrorDetails::NoPlatform)
        }
    }
}
