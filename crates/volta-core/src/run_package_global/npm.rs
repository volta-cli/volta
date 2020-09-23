use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::parser::CommandArg;
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;

/// Build an `Executor` for npm
///
/// If the command is a global install or uninstall and we have a default platform available, then
/// we will use custom logic to ensure that the package is correctly installed / uninstalled in the
/// Volta directory.
///
/// If the command is _not_ a global install / uninstall or we don't have a default platform, then
/// we will allow npm to execute the command as usual.
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    if let CommandArg::Global(cmd) = CommandArg::for_npm(args) {
        if let Some(default_platform) = session.default_platform()? {
            return cmd.executor(default_platform);
        }
    }

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
