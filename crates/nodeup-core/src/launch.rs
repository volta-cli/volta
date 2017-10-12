use std::env;
use std::env::ArgsOs;
use std::ffi::{OsString, OsStr};
use std::process::Command;

use config::{self, Config, Version};
use install;
use path;

pub fn prepare() -> OsString {
    // FIXME: handle None
    let Config { node: Version::Public(version) } = config::read().unwrap();

    install::by_version(&version);

    path::for_version(&version)
}

/**
 * Produces a pair containing the executable name (as passed in the first
 * element of `argv`) and the command-line arguments (as found in the rest
 * of `argv`).
 */
fn split_command() -> (OsString, ArgsOs) {
    let mut args = env::args_os();
    let arg0 = args.next().unwrap();
    (arg0, args)
}

pub fn binary(path_var: &OsStr) -> Command {
    let (exe, args) = split_command();

    // FIXME: at least in unix, use exec instead
    let mut command = Command::new(&exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(windows)]
pub fn script(path_var: &OsStr) -> Command {
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
pub fn script(path_var: &OsStr) -> Command {
    unimplemented!()
}
