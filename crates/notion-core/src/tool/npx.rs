use std::env::ArgsOs;
use std::ffi::OsStr;
use std::process::Command;

use super::{command_for, Tool};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};
use crate::version::VersionSpec;

use notion_fail::{throw, Fallible};

/// Represents a `npx` executable.
pub struct Npx(Command);

impl Tool for Npx {
    type Arguments = ArgsOs;

    fn new(args: ArgsOs, session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Npx);

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;

            // npx was only included with npm 5.2.0 and higher. If the npm version is less than that, we
            // should include a helpful error message
            let required_npm = VersionSpec::parse_version("5.2.0")?;
            if image.node.npm >= required_npm {
                Ok(Npx(command_for(OsStr::new("npx"), args, &image.path()?)))
            } else {
                throw!(ErrorDetails::NpxNotAvailable {
                    version: image.node.npm.to_string()
                });
            }
        } else {
            // Using 'Node' as the tool name since the npx version is derived from the Node version
            // This way the error message will prompt the user to add 'Node' to their toolchain, instead of 'npx'
            throw!(ErrorDetails::NoSuchTool {
                tool: "Node".to_string()
            });
        }
    }

    fn command(self) -> Command {
        self.0
    }
}
