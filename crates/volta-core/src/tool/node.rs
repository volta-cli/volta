use std::ffi::{OsStr, OsString};

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use volta_fail::Fallible;

pub(super) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Node);

    match session.current_platform()? {
        Some(ref platform) => {
            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("node"), args, &path))
        }
        None => ToolCommand::passthrough(OsStr::new("node"), args, ErrorDetails::NoPlatform),
    }
}
