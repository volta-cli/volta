use std::ffi::{OsStr, OsString};

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};
use crate::source::Source;

use log::debug;
use volta_fail::Fallible;

pub(super) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Node);

    match session.current_platform()? {
        Some(platform) => {
            match platform.source() {
                Source::Project => debug!("Using node@{} from project platform", platform.node()),
                Source::User => debug!("Using node@{} from user default platform", platform.node()),
            };

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("node"), args, &path))
        }
        None => {
            debug!("Could not find platform, delegating to system");
            ToolCommand::passthrough(OsStr::new("node"), args, ErrorDetails::NoPlatform)
        }
    }
}
