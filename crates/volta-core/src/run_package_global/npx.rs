use std::ffi::OsString;

use super::executor::{ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;
use crate::version::parse_version;

/// Build a `ToolCommand` for npx
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<ToolCommand> {
    let platform = Platform::current(session)?;

    Ok(ToolCommand::new("npx", args, platform, ToolKind::Npx))
}

/// Determine the execution context (PATH and failure error message) for npx
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;

            // npx was only included with npm 5.2.0 and higher. If the npm version is lower
            // that that, we should include a helpful error message
            let required_npm = parse_version("5.2.0")?;
            let active_npm = image.resolve_npm()?;
            if active_npm.value < required_npm {
                return Err(ErrorKind::NpxNotAvailable {
                    version: active_npm.value.to_string(),
                }
                .into());
            }

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
