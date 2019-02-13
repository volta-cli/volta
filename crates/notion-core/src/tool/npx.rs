use std::env::ArgsOs;
use std::ffi::OsStr;
use std::process::Command;

use failure::Fail;

use super::{command_for, NoSuchToolError, Tool};
use crate::session::{ActivityKind, Session};
use crate::version::VersionSpec;

use notion_fail::{throw, ExitCode, Fallible, NotionFail};
use notion_fail_derive::*;

/// Represents a `npx` executable.
pub struct Npx(Command);

#[derive(Debug, Fail, NotionFail)]
#[fail(
    display = r#"
'npx' is only available with npm >= 5.2.0

This project is configured to use version {} of npm."#,
    version
)]
#[notion_fail(code = "ExecutableNotFound")]
struct NpxNotAvailableError {
    version: String,
}

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
                Ok(Self::from_components(
                    OsStr::new("npx"),
                    args,
                    &image.path()?,
                ))
            } else {
                throw!(NpxNotAvailableError {
                    version: image.node.npm.to_string()
                });
            }
        } else {
            // Using 'Node' as the tool name since the npx version is derived from the Node version
            // This way the error message will prompt the user to add 'Node' to their toolchain, instead of 'npx'
            throw!(NoSuchToolError {
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
