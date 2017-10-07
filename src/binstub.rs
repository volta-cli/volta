extern crate nodeup_core;

use std::convert::AsRef;
use std::env;
use std::env::ArgsOs;
use std::ffi::{OsString, OsStr};
use std::path::PathBuf;
use std::iter::Extend;
use std::process::Command;

use nodeup_core::config::{Config, Version};

/**
 * Produces a pair containing the executable name (as passed in the first
 * element of `argv`) and the command-line arguments (as found in the rest
 * of `argv`).
 */
fn command_and_args() -> Option<(OsString, ArgsOs)> {
    let mut args = env::args_os();
    args.next().map(|arg0| { (arg0, args) })
}

/**
 * Produce a modified version of the current `PATH` environment varible that
 * will find Node.js executables in the installation directory for the given
 * version of Node instead of in the nodeup binstubs directory.
 */
fn instantiate_path<T: AsRef<OsStr>>(current: &T, version: &str) -> OsString {
    let toolchain_dir = &nodeup_core::config::toolchain_dir().unwrap();
    let split = env::split_paths(current).filter(|s| { s != toolchain_dir });
    let mut path_vec: Vec<PathBuf> = Vec::new();
    path_vec.push(nodeup_core::config::node_version_dir(version).map(|root| root.join("bin")).unwrap());
    path_vec.extend(split);
    env::join_paths(path_vec.iter()).unwrap()
}

fn main() {
    // FIXME: handle None
    let Config { node: Version::Public(version) } = nodeup_core::config::read().unwrap();

    nodeup_core::install::by_version(&version);

    let path_var = instantiate_path(&env::var_os("PATH").unwrap(), &version);

    let (exe, args) = command_and_args().unwrap();

    // FIXME: at least in unix, use exec instead
    let status = Command::new(&exe)
        .args(args)
        .env("PATH", path_var)
        .status()
        .unwrap();

    println!("process exited with {}", status);
    // FIXME: exit with the same status code
}
