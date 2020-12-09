//! Types and helpers for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::iter::empty;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::{CliPlatform, Sourced, System};
use crate::session::Session;
use crate::signal::pass_control_to_shim;
use crate::style::tool_version;
use log::debug;

pub mod binary;
pub mod node;
pub mod npm;
pub mod npx;
pub mod yarn;

const VOLTA_BYPASS: &str = "VOLTA_BYPASS";
const UNSAFE_GLOBAL: &str = "VOLTA_UNSAFE_GLOBAL";

pub fn execute_shim(session: &mut Session) -> Fallible<ExitStatus> {
    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;
    let envs = empty::<(String, String)>();

    execute_tool(&exe, args, envs, CliPlatform::default(), session)
}

pub fn execute_tool<A, S, E, K, V>(
    exe: &OsStr,
    args: A,
    envs: E,
    cli: CliPlatform,
    session: &mut Session,
) -> Fallible<ExitStatus>
where
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut command = if env::var_os(VOLTA_BYPASS).is_some() {
        ToolCommand::passthrough(
            &exe,
            ErrorKind::BypassError {
                command: exe.to_string_lossy().to_string(),
            },
        )?
    } else {
        match exe.to_str() {
            Some("volta-shim") => return Err(ErrorKind::RunShimDirectly.into()),
            Some("node") => node::command(cli, session)?,
            Some("npm") => npm::command(cli, session)?,
            Some("npx") => npx::command(cli, session)?,
            Some("yarn") => yarn::command(cli, session)?,
            _ => binary::command(exe, cli, session)?,
        }
    };

    command.args(args);
    command.envs(envs);

    pass_control_to_shim();
    command.status()
}

/// Process builder for launching a Volta-managed tool
///
/// This is a thin wrapper around std::process::Command, providing a few QoL improvements:
///
/// * `ErrorKind` error type on `status` and `output` methods, determined based on the context
/// * Helper methods for constructing a type with the appropriate context
pub(crate) struct ToolCommand {
    /// The wrapped Command
    command: Command,

    /// The Volta error with which to wrap any failure.
    ///
    /// This allows us to call out to the system for the pass-through behavior, but still
    /// show a friendly error message for cases where the user needs to select a Node version
    on_failure: ErrorKind,
}

impl ToolCommand {
    /// Build a ToolCommand that is directly calling a tool in the Volta directory
    fn direct(exe: &OsStr, path_var: &OsStr) -> Self {
        ToolCommand {
            command: command_with_path(exe, path_var),
            on_failure: ErrorKind::BinaryExecError,
        }
    }

    /// Build a ToolCommand that is calling a binary in the current project's `node_modules/bin`
    fn project_local(exe: &OsStr, path_var: &OsStr) -> Self {
        ToolCommand {
            command: command_with_path(exe, path_var),
            on_failure: ErrorKind::ProjectLocalBinaryExecError {
                command: exe.to_string_lossy().to_string(),
            },
        }
    }

    /// Build a ToolCommand that is calling a command that Volta couldn't find
    ///
    /// This will allow the existing system to resolve the tool, if possible. If that still fails,
    /// then we show `default_error` as the friendly error to the user, directing them how to
    /// resolve the issue (e.g. run `volta install node` to enable `node`)
    fn passthrough(exe: &OsStr, default_error: ErrorKind) -> Fallible<Self> {
        let path = System::path()?;
        Ok(ToolCommand {
            command: command_with_path(exe, &path),
            on_failure: default_error,
        })
    }

    /// Add a single argument to the Command.
    ///
    /// The new argument will be added to the end of the current argument list
    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut ToolCommand {
        self.command.arg(arg);
        self
    }

    /// Add multiple arguments to the Command.
    ///
    /// New arguments will be added at the end of the current argument list
    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut ToolCommand
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item = S>,
    {
        self.command.args(args);
        self
    }

    /// Adds or updates multiple environment variables for the Command
    pub(crate) fn envs<E, K, V>(&mut self, envs: E) -> &mut ToolCommand
    where
        E: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(envs);
        self
    }

    /// Set the current working directory for the Command
    pub(crate) fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut ToolCommand {
        self.command.current_dir(dir);
        self
    }

    /// Execute the command, returning its status
    ///
    /// Any failures will be wrapped with the Error value in `on_failure`
    pub(crate) fn status(mut self) -> Fallible<ExitStatus> {
        self.command.status().with_context(|| self.on_failure)
    }

    /// Execute the command, returning all of its output to the caller
    ///
    /// Any failures will be wrapped with the Error value in `on_failure`
    pub(crate) fn output(mut self) -> Fallible<Output> {
        self.command.output().with_context(|| self.on_failure)
    }
}

impl fmt::Debug for ToolCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.command)
    }
}

pub fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
    args.next()
        .and_then(|arg0| Path::new(&arg0).file_name().map(tool_name_from_file_name))
        .ok_or_else(|| ErrorKind::CouldNotDetermineTool.into())
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

/// Create a command in the given context by setting the `PATH` environment variable
fn command_with_path(exe: &OsStr, path_var: &OsStr) -> Command {
    let mut command = create_command(exe);
    command.env("PATH", path_var);
    command
}

/// Determine if we should intercept global installs or not
///
/// Setting the VOLTA_UNSAFE_GLOBAL environment variable will disable interception of global installs
fn intercept_global_installs() -> bool {
    env::var_os(UNSAFE_GLOBAL).is_none()
}

/// Distinguish global `add` commands in npm or yarn from all others.
enum CommandArg {
    /// The command is a *global* add command.
    GlobalAdd(Option<OsString>),
    /// The command is a local, i.e. non-global, add command.
    NotGlobalAdd,
}

/// Write the tool version and source to the debug log
fn debug_tool_message<T>(tool: &str, version: &Sourced<T>)
where
    T: std::fmt::Display + Sized,
{
    debug!(
        "Using {} from {} configuration",
        tool_version(tool, &version.value),
        version.source,
    )
}
