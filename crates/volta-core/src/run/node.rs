use std::env;
use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform, RECURSION_ENV_VAR};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::{ActivityKind, Session};

/// Build a `ToolCommand` for Node
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Node);
    // Don't re-evaluate the platform if this is a recursive call
    let platform = match env::var_os(RECURSION_ENV_VAR) {
        Some(_) => None,
        None => Platform::current(session)?,
    };

    Ok(ToolCommand::new("node", args, platform, ToolKind::Node).into())
}

/// Determine the execution context (PATH and failure error message) for Node
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;
            let path = image.path()?;
            debug_active_image(&image);

            Ok((path, ErrorKind::BinaryExecError))
        }
        None => {
            let path = System::path()?;
            debug_no_platform();
            Ok((path, ErrorKind::NoPlatform))
        }
    }
}
