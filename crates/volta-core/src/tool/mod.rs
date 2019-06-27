//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::command::create_command;
use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::platform::System;
use crate::session::Session;
use volta_fail::{Fallible, ResultExt};

mod binary;
mod node;
mod npm;
mod npx;
mod spec;
mod version;
mod yarn;

pub use spec::ToolSpec;
pub use version::ToolVersion;

/// Distinguish global `add` commands in npm or yarn from all others.
enum CommandArg {
    /// The command is a *global* add command.
    GlobalAdd(Option<OsString>),
    /// The command is a local, i.e. non-global, add command.
    NotGlobalAdd,
}

pub fn execute_tool(session: &mut Session) -> Fallible<ExitStatus> {
    path::ensure_volta_dirs_exist()?;

    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;

    let command = match &exe.to_str() {
        Some("node") => node::command(args, session)?,
        Some("npm") => npm::command(args, session)?,
        Some("npx") => npx::command(args, session)?,
        Some("yarn") => yarn::command(args, session)?,
        _ => binary::command(exe, args, session)?,
    };

    command.exec()
}

/// Represents the command to execute a tool
struct ToolCommand {
    command: Command,
    error: ErrorDetails,
}

impl ToolCommand {
    fn direct<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand {
            command: command_for(exe, args, path_var),
            error: ErrorDetails::BinaryExecError,
        }
    }

    fn project_local<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand {
            command: command_for(exe, args, path_var),
            error: ErrorDetails::ProjectLocalBinaryExecError {
                command: exe.to_string_lossy().to_string(),
            },
        }
    }

    fn passthrough<A>(exe: &OsStr, args: A, default_error: ErrorDetails) -> Fallible<Self>
    where
        A: IntoIterator<Item = OsString>,
    {
        let path = System::path()?;
        Ok(ToolCommand {
            command: command_for(exe, args, &path),
            error: default_error,
        })
    }

    fn exec(mut self) -> Fallible<ExitStatus> {
        self.command.status().with_context(|_| self.error)
    }
}

fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
    args.nth(0)
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or(ErrorDetails::CouldNotDetermineTool.into())
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix
    // We need to remove that to get the raw tool name
    match file_name.to_str() {
        Some(file) => OsString::from(file.trim_end_matches(".exe")),
        None => OsString::from(file_name),
    }
}

fn command_for<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Command
where
    A: IntoIterator<Item = OsString>,
{
    let mut command = create_command(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

fn intercept_global_installs() -> bool {
    // We should only intercept global installs if the VOLTA_UNSAFE_GLOBAL variable is not set
    env::var_os(UNSAFE_GLOBAL).is_none()
}
