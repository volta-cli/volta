use std::env::{args_os, ArgsOs};
use std::ffi::{OsString, OsStr};
use std::process::{Command, ExitStatus, exit};
use std::path::Path;

use project::Project;
use install;
use env;

fn exec_with<F: FnOnce() -> ::Result<Command>>(get_command: F) -> ::Result<ExitStatus> {
    let mut command = get_command()?;
    let status = command.status()?;
    Ok(status)
}

fn exec<F: FnOnce() -> ::Result<Command>>(get_command: F) -> ! {
    match exec_with(get_command) {
        Ok(status) if status.success() => {
            exit(0);
        }
        Ok(status) => {
            // FIXME: if None, in unix, find out the signal
            exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            ::display_error(err);
            exit(1);
        }
    }
}

pub fn prepare() -> ::Result<OsString> {
    let mut project = Project::for_current_dir()?.unwrap();
    let lockfile = project.lockfile()?;
    let version = &lockfile.node.version;
    install::by_version(version)?;
    Ok(env::path_for(version))
}

/**
 * Produces a pair containing the executable name (as passed in the first
 * element of `argv`) and the command-line arguments (as found in the rest
 * of `argv`).
 */
fn split_command() -> (OsString, ArgsOs) {
    let mut args = args_os();
    // FIXME: make an error kind for this case
    let arg0 = Path::new(&args.next().unwrap()).file_name().unwrap().to_os_string();
    (arg0, args)
}

fn binary_command(path_var: &OsStr) -> Command {
    let (exe, args) = split_command();

    // FIXME: at least in unix, use exec instead
    let mut command = Command::new(&exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(windows)]
fn script_command(path_var: &OsStr) -> Command {
    let (exe, args) = split_command();

    // See: https://github.com/rust-lang/rust/issues/42791
    let mut command = Command::new("cmd.exe");
    command.arg("/C");
    command.arg(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(not(windows))]
fn script_command(path_var: &OsStr) -> Command {
    unimplemented!()
}

pub fn binary() -> ! {
    exec(|| { Ok(binary_command(&prepare()?)) })
}

pub fn script() -> ! {
    exec(|| { Ok(script_command(&prepare()?)) })
}
