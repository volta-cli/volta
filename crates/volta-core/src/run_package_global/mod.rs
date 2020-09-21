use std::env;
use std::env::ArgsOs;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::ExitStatus;

use crate::error::{ErrorKind, Fallible};
use crate::platform::{CliPlatform, Image, Sourced};
use crate::session::Session;
use crate::tool;
use log::debug;
use semver::Version;

pub mod binary;
mod executor;
mod node;
mod npm;
mod npx;
mod yarn;

const VOLTA_BYPASS: &str = "VOLTA_BYPASS";

/// Execute a shim command, based on the command-line arguments to the current process
pub fn execute_shim(session: &mut Session) -> Fallible<ExitStatus> {
    let mut native_args = env::args_os();
    let exe = get_tool_name(&mut native_args)?;
    let args: Vec<_> = native_args.collect();

    get_executor(&exe, &args, session)?.execute(session)
}

/// Execute a tool with the provided arguments
pub fn execute_tool<E, K, V>(
    exe: &OsStr,
    args: &[OsString],
    envs: E,
    cli: CliPlatform,
    session: &mut Session,
) -> Fallible<ExitStatus>
where
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut runner = get_executor(exe, args, session)?;
    runner.cli_platform(cli);
    runner.envs(envs);

    runner.execute(session)
}

/// Get the appropriate Tool command, based on the requested executable and arguments
fn get_executor(
    exe: &OsStr,
    args: &[OsString],
    session: &mut Session,
) -> Fallible<executor::Executor> {
    if env::var_os(VOLTA_BYPASS).is_some() {
        Ok(executor::ToolCommand::new(
            exe,
            args,
            None,
            executor::ToolKind::Bypass(exe.to_string_lossy().to_string()),
        )
        .into())
    } else {
        match exe.to_str() {
            Some("volta-shim") => Err(ErrorKind::RunShimDirectly.into()),
            Some("node") => node::command(args, session),
            Some("npm") => npm::command(args, session),
            Some("npx") => npx::command(args, session),
            Some("yarn") => yarn::command(args, session),
            _ => binary::command(exe, args, session),
        }
    }
}

/// Determine the name of the command to run by inspecting the first argument to the active process
fn get_tool_name(args: &mut ArgsOs) -> Fallible<OsString> {
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

#[inline]
fn debug_no_platform() {
    debug!("Could not find Volta-managed platform, delegating to system");
}

#[inline]
fn debug_active_image(image: &Image) {
    debug!(
        "Active Image:
    Node: {}
    npm: {}
    Yarn: {}",
        format_tool_version(&image.node),
        image
            .resolve_npm()
            .ok()
            .as_ref()
            .map(format_tool_version)
            .unwrap_or_else(|| "Bundled with Node".into()),
        image
            .yarn
            .as_ref()
            .map(format_tool_version)
            .unwrap_or_else(|| "None".into()),
    )
}

fn format_tool_version(version: &Sourced<Version>) -> String {
    format!("{} from {} configuration", version.value, version.source)
}

/// Distinguish global `add` commands in npm or yarn from all others
enum CommandArg {
    /// The command is a global add command
    GlobalAdd(tool::Spec),
    /// The command is a global remove command
    GlobalRemove(tool::Spec),
    /// The command is *not* a global command
    NotGlobal,
}
