//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};

use failure::Fail;

use crate::env::UNSAFE_GLOBAL;
use crate::session::Session;
use crate::style;
use crate::version::VersionSpec;
use notion_fail::{throw, ExitCode, Fallible, NotionError, NotionFail, ResultExt};
use notion_fail_derive::*;

mod binary;
mod node;
mod npm;
mod npx;
mod yarn;

use self::binary::{Binary, BinaryArgs};
use self::node::Node;
use self::npm::Npm;
use self::npx::Npx;
use self::yarn::Yarn;

fn display_tool_error(err: &NotionError) {
    style::display_error(style::ErrorContext::Shim, err);
}

pub enum ToolSpec {
    Node(VersionSpec),
    Yarn(VersionSpec),
    Npm(VersionSpec),
    Package(String, VersionSpec),
}

impl ToolSpec {
    pub fn from_str(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => ToolSpec::Node(version),
            "yarn" => ToolSpec::Yarn(version),
            "npm" => ToolSpec::Npm(version),
            package => ToolSpec::Package(package.to_string(), version),
        }
    }
}

impl Debug for ToolSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &ToolSpec::Node(ref version) => format!("node version {}", version),
            &ToolSpec::Yarn(ref version) => format!("yarn version {}", version),
            &ToolSpec::Npm(ref version) => format!("npm version {}", version),
            &ToolSpec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}

impl Display for ToolSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &ToolSpec::Node(ref version) => format!("node version {}", version),
            &ToolSpec::Yarn(ref version) => format!("yarn version {}", version),
            &ToolSpec::Npm(ref version) => format!("npm version {}", version),
            &ToolSpec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{}", error)]
#[notion_fail(code = "ExecutionFailure")]
pub(crate) struct BinaryExecError {
    pub(crate) error: String,
}

impl BinaryExecError {
    pub(crate) fn from_io_error(error: &io::Error) -> Self {
        if let Some(inner_err) = error.get_ref() {
            BinaryExecError {
                error: inner_err.to_string(),
            }
        } else {
            BinaryExecError {
                error: error.to_string(),
            }
        }
    }
}

pub fn execute_tool(session: &mut Session) -> Fallible<ExitStatus> {
    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;

    match &exe.to_str() {
        Some("node") => Node::new(args, session)?.exec(session),
        Some("npm") => Npm::new(args, session)?.exec(session),
        Some("npx") => Npx::new(args, session)?.exec(session),
        Some("yarn") => Yarn::new(args, session)?.exec(session),
        _ => Binary::new(
            BinaryArgs {
                executable: exe,
                args,
            },
            session,
        )?
        .exec(session),
    }
}

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    type Arguments;

    /// Constructs a new instance.
    fn new(args: Self::Arguments, session: &mut Session) -> Fallible<Self>;

    /// Constructs a new instance, using the specified command-line and `PATH` variable.
    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    /// Extracts the `Command` from this tool.
    fn command(self) -> Command;

    /// Perform any tasks which must be run after the tool runs but before exiting.
    fn finalize(_session: &Session, _maybe_status: &io::Result<ExitStatus>) {}

    /// Delegates the current process to this tool.
    fn exec(self, session: &Session) -> Fallible<ExitStatus> {
        let mut command = self.command();
        let status = command.status();
        Self::finalize(session, &status);
        status.with_context(BinaryExecError::from_io_error)
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Tool name could not be determined")]
#[notion_fail(code = "UnknownError")]
struct CouldNotDetermineTool;

fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
    let opt = args
        .next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name));
    if let Some(tool_name) = opt {
        Ok(tool_name)
    } else {
        throw!(CouldNotDetermineTool);
    }
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows PowerShell, the file name includes the .exe suffix
    // We need to remove that to get the raw tool name
    let mut result = OsString::new();
    match file_name.to_str() {
        Some(file) => {
            result.push(file.trim_end_matches(".exe"));
        }
        None => {
            result.push(file_name);
        }
    }

    result
}

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[derive(Debug, Fail, NotionFail)]
#[fail(
    display = r#"
No {} version selected.

See `notion help pin` for help adding {} to a project toolchain.

See `notion help install` for help adding {} to your personal toolchain."#,
    tool, tool, tool
)]
#[notion_fail(code = "NoVersionMatch")]
struct NoSuchToolError {
    tool: String,
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = r#"
Global package installs are not recommended.

Consider using `notion install` to add a package to your toolchain (see `notion help install` for more info)."#)]
#[notion_fail(code = "InvalidArguments")]
struct NoGlobalInstallError;

fn intercept_global_installs() -> bool {
    if cfg!(feature = "intercept-globals") {
        // We should only intercept global installs if the NOTION_UNSAFE_GLOBAL variable is not set
        env::var_os(UNSAFE_GLOBAL).is_none()
    } else {
        false
    }
}
