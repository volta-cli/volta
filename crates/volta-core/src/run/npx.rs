use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::platform::Source;
use crate::session::{ActivityKind, Session};
use crate::style::tool_version;
use crate::version::VersionSpec;

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
    session.add_event_start(ActivityKind::Npx);

    match session.current_platform()? {
        Some(platform) => {
            let image = platform.checkout(session)?;

            // npx was only included with npm 5.2.0 and higher. If the npm version is less than that, we
            // should include a helpful error message
            let required_npm = VersionSpec::parse_version("5.2.0")?;
            if image.node().npm >= required_npm {
                let source = match image.source() {
                    Source::Project | Source::ProjectNodeDefaultYarn => "project",
                    Source::Default => "default",
                };
                let version = tool_version("npx", &image.node().npm);
                debug!("Using {} from {} configuration", version, source);

                let path = image.path()?;
                Ok(ToolCommand::direct(
                    OsStr::new("npx"),
                    args,
                    &path,
                    current_dir,
                ))
            } else {
                Err(ErrorDetails::NpxNotAvailable {
                    version: image.node().npm.to_string(),
                }
                .into())
            }
        }
        None => {
            debug!("Could not find Volta-managed npx, delegating to system");
            ToolCommand::passthrough(
                OsStr::new("npx"),
                args,
                ErrorDetails::NoPlatform,
                current_dir,
            )
        }
    }
}
