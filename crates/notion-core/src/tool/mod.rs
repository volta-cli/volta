//! Traits and types for executing command-line tools.

use std::env::{self, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::session::{ActivityKind, Session};
use crate::style;
use crate::version::VersionSpec;
use notion_fail::{throw, ExitCode, FailExt, Fallible, NotionError};

mod binary;
mod node;
mod npm;
mod npx;
mod script;
mod yarn;

pub use self::binary::Binary;
pub use self::node::Node;
pub use self::npm::Npm;
pub use self::npx::Npx;
pub use self::script::Script;
pub use self::yarn::Yarn;

fn display_error(err: &NotionError) {
    if err.is_user_friendly() {
        style::display_error(style::ErrorContext::Shim, err);
    } else {
        style::display_unknown_error(style::ErrorContext::Shim, err);
    }
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

fn binary_exec_error(error: &io::Error) -> ErrorDetails {
    if let Some(inner_err) = error.get_ref() {
        ErrorDetails::BinaryExecError {
            error: inner_err.to_string(),
        }
    } else {
        ErrorDetails::BinaryExecError {
            error: error.to_string(),
        }
    }
}

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    fn launch() -> ! {
        let mut session = Session::new();

        session.add_event_start(ActivityKind::Tool);

        let tool_result = path::ensure_notion_dirs_exist().and_then(|_| Self::new(&mut session));
        match tool_result {
            Ok(tool) => {
                tool.exec(session);
            }
            Err(err) => {
                display_error(&err);
                session.add_event_error(ActivityKind::Tool, &err);
                session.exit(ExitCode::ExecutionFailure);
            }
        }
    }

    /// Constructs a new instance.
    fn new(_: &mut Session) -> Fallible<Self>;

    /// Constructs a new instance, using the specified command-line and `PATH` variable.
    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    /// Extracts the `Command` from this tool.
    fn command(self) -> Command;

    /// Perform any tasks which must be run after the tool runs but before exiting.
    fn finalize(_session: &Session, _maybe_status: &io::Result<ExitStatus>) {}

    /// Delegates the current process to this tool.
    fn exec(self, mut session: Session) -> ! {
        let mut command = self.command();
        let status = command.status();
        Self::finalize(&session, &status);
        match status {
            Ok(status) if status.success() => {
                session.add_event_end(ActivityKind::Tool, ExitCode::Success);
                session.exit(ExitCode::Success);
            }
            Ok(status) => {
                // ISSUE (#36): if None, in unix, find out the signal
                let code = status.code().unwrap_or(1);
                session.add_event_tool_end(ActivityKind::Tool, code);
                session.exit_tool(code);
            }
            Err(err) => {
                let notion_err = err.with_context(binary_exec_error);
                display_error(&notion_err);
                session.add_event_error(ActivityKind::Tool, &notion_err);
                session.exit(ExitCode::ExecutionFailure);
            }
        }
    }
}

#[cfg(unix)]
fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(windows)]
fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    // Several of the node utilities are implemented as `.bat` or `.cmd` files
    // When executing those files with `Command`, we need to call them with:
    //    cmd.exe /C <COMMAND> <ARGUMENTS>
    // Instead of: <COMMAND> <ARGUMENTS>
    // See: https://github.com/rust-lang/rust/issues/42791 For a longer discussion
    let mut command = Command::new("cmd.exe");
    command.arg("/C");
    command.arg(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

fn arg0(args: &mut ArgsOs) -> Fallible<OsString> {
    let opt = args
        .next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name));
    if let Some(file_name) = opt {
        Ok(file_name)
    } else {
        throw!(ErrorDetails::CouldNotDetermineTool);
    }
}

#[cfg(unix)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    file_name.to_os_string()
}

#[cfg(windows)]
fn tool_name_from_file_name(file_name: &OsStr) -> OsString {
    // On Windows Powershell, the file name includes the .exe suffix
    // We need to remove that, because many of the tools are not .exe files
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

fn intercept_global_installs() -> bool {
    if cfg!(feature = "intercept-globals") {
        // We should only intercept global installs if the NOTION_UNSAFE_GLOBAL variable is not set
        env::var_os(UNSAFE_GLOBAL).is_none()
    } else {
        false
    }
}
