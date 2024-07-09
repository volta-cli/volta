use std::env;
use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform, RECURSION_ENV_VAR};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::{ActivityKind, Session};
use node_semver::Version;
use once_cell::sync::Lazy;

static REQUIRED_NPM_VERSION: Lazy<Version> = Lazy::new(|| Version {
    major: 5,
    minor: 2,
    patch: 0,
    build: vec![],
    pre_release: vec![],
});

/// Build a `ToolCommand` for npx
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Npx);
    // Don't re-evaluate the context if this is a recursive call
    let platform = match env::var_os(RECURSION_ENV_VAR) {
        Some(_) => None,
        None => Platform::current(session)?,
    };

    Ok(ToolCommand::new("npx", args, platform, ToolKind::Npx).into())
}

/// Determine the execution context (PATH and failure error message) for npx
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;

            // If the npm version is lower than the minimum required, we can show a helpful error
            // message instead of a 'command not found' error.
            let active_npm = image.resolve_npm()?;
            if active_npm.value < *REQUIRED_NPM_VERSION {
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
