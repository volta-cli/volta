//! Traits and types for executing command-line tools.

use std::env::{self, args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::env::UNSAFE_GLOBAL;
use crate::error::ErrorDetails;
use crate::platform::System;
use crate::session::Session;
use crate::version::VersionSpec;
use notion_fail::{Fallible, ResultExt};

mod binary;
mod node;
mod npm;
mod npx;
mod yarn;

use self::binary::binary_command;
use self::node::node_command;
use self::npm::npm_command;
use self::npx::npx_command;
use self::yarn::yarn_command;

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

pub fn execute_tool(session: &mut Session) -> Fallible<ExitStatus> {
    let mut args = args_os();
    let exe = get_tool_name(&mut args)?;

    let command = match &exe.to_str() {
        Some("node") => node_command(args, session)?,
        Some("npm") => npm_command(args, session)?,
        Some("npx") => npx_command(args, session)?,
        Some("yarn") => yarn_command(args, session)?,
        _ => binary_command(exe, args, session)?,
    };

    command.exec()
}

/// Represents the command to execute a tool
enum ToolCommand {
    Direct(Command),
    Passthrough(Command, ErrorDetails),
}

impl ToolCommand {
    fn direct<A>(exe: &OsStr, args: A, path_var: &OsStr) -> Self
    where
        A: IntoIterator<Item = OsString>,
    {
        ToolCommand::Direct(command_for(exe, args, path_var))
    }

    fn passthrough<A>(exe: &OsStr, args: A, default_error: ErrorDetails) -> Fallible<Self>
    where
        A: IntoIterator<Item = OsString>,
    {
        let path = System::path()?;
        Ok(ToolCommand::Passthrough(
            command_for(exe, args, &path),
            default_error,
        ))
    }

    fn exec(self) -> Fallible<ExitStatus> {
        match self {
            ToolCommand::Direct(mut command) => command
                .status()
                .with_context(|_| ErrorDetails::BinaryExecError),
            ToolCommand::Passthrough(mut command, error) => {
                command.status().with_context(|_| error)
            }
        }
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

fn command_for<A: IntoIterator<Item = OsString>>(
    exe: &OsStr,
    args: A,
    path_var: &OsStr,
) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

fn intercept_global_installs() -> bool {
    // We should only intercept global installs if the NOTION_UNSAFE_GLOBAL variable is not set
    env::var_os(UNSAFE_GLOBAL).is_none()
}
