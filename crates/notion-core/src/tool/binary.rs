use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::Command;

use failure::Fail;

use super::{arg0, command_for, NoSuchToolError, Tool};
use crate::path;
use crate::session::{ActivityKind, Session};
use notion_fail::{throw, ExitCode, Fallible, NotionFail};
use notion_fail_derive::*;

/// Represents a delegated binary executable.
pub struct Binary(Command);

impl Tool for Binary {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Binary);

        let mut args = args_os();
        let exe = arg0(&mut args)?;

        // first try to use the project toolchain
        if let Some(project) = session.project()? {
            // check if the executable is a direct dependency
            if project.has_direct_bin(&exe)? {
                // use the full path to the file
                let mut path_to_bin = project.local_bin_dir();
                path_to_bin.push(&exe);

                // if we're in a pinned project, use the project's platform.
                if let Some(ref platform) = session.project_platform()? {
                    let image = platform.checkout(session)?;
                    return Ok(Self::from_components(
                        &path_to_bin.as_os_str(),
                        args,
                        &image.path()?,
                    ));
                }

                // otherwise use the user platform.
                if let Some(ref platform) = session.user_platform()? {
                    let image = platform.checkout(session)?;
                    return Ok(Self::from_components(
                        &path_to_bin.as_os_str(),
                        args,
                        &image.path()?,
                    ));
                }

                // if there's no user platform selected, fail.
                throw!(NoSuchToolError {
                    tool: "Node".to_string()
                });
            }
        }

        // next try to use the user toolchain
        if let Some(ref platform) = session.user_platform()? {
            // use the full path to the binary
            // ISSUE (#160): Look up the platform image bound to the user tool.
            let image = platform.checkout(session)?;
            let node_str = image.node.runtime.to_string();
            let npm_str = image.node.npm.to_string();
            let mut third_p_bin_dir = path::node_image_3p_bin_dir(&node_str, &npm_str)?;
            third_p_bin_dir.push(&exe);
            return Ok(Self::from_components(
                &third_p_bin_dir.as_os_str(),
                args,
                &image.path()?,
            ));
        };

        // at this point, there is no project or user toolchain
        // the user is executing a Notion shim that doesn't have a way to execute it
        throw!(NoToolChainError::for_shim(
            exe.to_string_lossy().to_string()
        ));
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Binary(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "No toolchain available to run shim {}", shim_name)]
#[notion_fail(code = "ExecutionFailure")]
pub(crate) struct NoToolChainError {
    shim_name: String,
}

impl NoToolChainError {
    pub(crate) fn for_shim(shim_name: String) -> NoToolChainError {
        NoToolChainError { shim_name }
    }
}
