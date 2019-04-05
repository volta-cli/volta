//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::path;
use crate::session::Session;
use crate::version::VersionSpec;
use notion_fail::{Fallible, ResultExt};

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

pub enum ToolSpec {
    Node(VersionSpec),
    Yarn(VersionSpec),
    Npm(VersionSpec),
    Package(String, VersionSpec),
}

impl ToolSpec {
    pub fn from_str_and_version(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => ToolSpec::Node(version),
            "yarn" => ToolSpec::Yarn(version),
            "npm" => ToolSpec::Npm(version),
            package => ToolSpec::Package(package.to_string(), version),
        }
    }

    pub fn install(&self, session: &mut Session) -> Fallible<()> {
        match self {
            ToolSpec::Node(version) => session.install_node(&version)?,
            ToolSpec::Yarn(version) => session.install_yarn(&version)?,
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => unimplemented!("Installing npm is not supported yet"),
            ToolSpec::Package(name, version) => {
                session.install_package(name.to_string(), &version)?;
            }
        }
        Ok(())
    }

    pub fn uninstall(&self, session: &mut Session) -> Fallible<()> {
        match self {
            ToolSpec::Node(_version) => unimplemented!("Uninstalling Node not supported yet"),
            ToolSpec::Yarn(_version) => unimplemented!("Uninstalling Yarn not supported yet"),
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => unimplemented!("Uninstalling Npm not supported yet"),
            ToolSpec::Package(name, _version) => {
                session.uninstall_package(name.to_string())?;
            }
        }
        Ok(())
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

pub fn execute_tool(session: &mut Session) -> Fallible<ExitStatus> {
    path::ensure_notion_dirs_exist()?;

    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;

    // There is some duplication in the calls to `.exec` here.
    // It's required because we can't create a single variable that holds
    // all the possible `Tool` implementations and fill it dynamically,
    // as they have different sizes and associated types.
    match &exe.to_str() {
        Some("node") => Node::new(args, session)?.exec(),
        Some("npm") => Npm::new(args, session)?.exec(),
        Some("npx") => Npx::new(args, session)?.exec(),
        Some("yarn") => Yarn::new(args, session)?.exec(),
        _ => Binary::new(
            BinaryArgs {
                executable: exe,
                args,
            },
            session,
        )?
        .exec(),
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

    /// Delegates the current process to this tool.
    fn exec(self) -> Fallible<ExitStatus> {
        let mut command = self.command();
        let status = command.status();
        status.with_context(binary_exec_error)
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

fn intercept_global_installs() -> bool {
    // We should only intercept global installs if the NOTION_UNSAFE_GLOBAL variable is not set
    env::var_os(UNSAFE_GLOBAL).is_none()
}
