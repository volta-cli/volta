use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;

/// Build a `ToolCommand` for npm
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    let platform = Platform::current(session)?;

    Ok(ToolCommand::new("npm", args, platform, ToolKind::Npm).into())
}

/// Determine the execution context (PATH and failure error message) for npm
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
