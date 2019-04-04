use std::env::ArgsOs;
use std::ffi::OsString;
use std::iter::once;
use std::process::Command;

use super::{command_for, Tool};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use notion_fail::{throw, Fallible};

/// Represents a delegated binary executable.
pub struct Binary(Command);

/// Represents the arguments needed for a binary executable
/// Both the executable name and the arguments to pass to it
pub struct BinaryArgs {
    pub executable: OsString,
    pub args: ArgsOs,
}

impl Tool for Binary {
    type Arguments = BinaryArgs;

    fn new(params: BinaryArgs, session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Binary);

        // first try to use the project toolchain
        if let Some(project) = session.project()? {
            // check if the executable is a direct dependency
            if project.has_direct_bin(&params.executable)? {
                // use the full path to the file
                let mut path_to_bin = project.local_bin_dir();
                path_to_bin.push(&params.executable);

                // if we're in a pinned project, use the project's platform.
                if let Some(ref platform) = session.project_platform()? {
                    let image = platform.checkout(session)?;
                    return Ok(Binary(command_for(
                        path_to_bin.as_os_str(),
                        params.args,
                        &image.path()?,
                    )));
                }

                // otherwise use the user platform.
                if let Some(ref platform) = session.user_platform()? {
                    let image = platform.checkout(session)?;
                    return Ok(Binary(command_for(
                        path_to_bin.as_os_str(),
                        params.args,
                        &image.path()?,
                    )));
                }

                // if there's no user platform selected, fail.
                throw!(ErrorDetails::NoSuchTool {
                    tool: "Node".to_string()
                });
            }
        }

        // try to use the user toolchain
        if let Some(user_tool) = session.get_user_tool(&params.executable)? {
            let tool_path = user_tool.bin_path.into_os_string();
            let cmd = match user_tool.loader {
                Some(exe) => command_for(
                    exe.as_ref(),
                    once(tool_path).chain(params.args),
                    &user_tool.image.path()?,
                ),
                None => command_for(&tool_path, params.args, &user_tool.image.path()?),
            };
            return Ok(Binary(cmd));
        }

        // at this point, there is no project or user toolchain
        // the user is executing a Notion shim that doesn't have a way to execute it
        throw!(ErrorDetails::NoToolChain {
            shim_name: params.executable.to_string_lossy().to_string(),
        });
    }

    fn command(self) -> Command {
        self.0
    }
}
