use std::ffi::OsStr;

use super::{debug_tool_message, ToolCommand};
use crate::error::ErrorDetails;
use crate::platform::{CliPlatform, Platform};
use crate::session::{ActivityKind, Session};
use crate::version::parse_version;

use log::debug;
use volta_fail::Fallible;

pub(crate) fn command(cli: CliPlatform, session: &mut Session) -> Fallible<ToolCommand> {
    session.add_event_start(ActivityKind::Npx);

    match Platform::with_cli(cli, session)? {
        Some(platform) => {
            let image = platform.checkout(session)?;

            // npx was only included with npm 5.2.0 and higher. If the npm version is less than that, we
            // should include a helpful error message
            let required_npm = parse_version("5.2.0")?;
            let active_npm = image.resolve_npm()?;
            if active_npm.value >= required_npm {
                let path = image.path()?;

                debug_tool_message("npx", &active_npm);
                Ok(ToolCommand::direct(OsStr::new("npx"), &path))
            } else {
                Err(ErrorDetails::NpxNotAvailable {
                    version: active_npm.value.to_string(),
                }
                .into())
            }
        }
        None => {
            debug!("Could not find Volta-managed npx, delegating to system");
            ToolCommand::passthrough(OsStr::new("npx"), ErrorDetails::NoPlatform)
        }
    }
}
