use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::Command;

use super::{arg0, command_for, Tool};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};
use crate::version::VersionSpec;

use notion_fail::{throw, Fallible};

/// Represents a `npx` executable.
pub struct Npx(Command);

impl Tool for Npx {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Npx);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;

            // npx was only included with Node >= 8.2.0. If less than that, we should include a helpful error message
            let required_node = VersionSpec::parse_requirements(">= 5.2.0")?;
            if required_node.matches(&image.node.npm) {
                Ok(Self::from_components(&exe, args, &image.path()?))
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

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Npx(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}
