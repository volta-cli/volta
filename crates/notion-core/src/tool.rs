use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::{Command, exit};
use std::path::Path;
use std::marker::Sized;

use session::Session;
use env;
use failure;
use style;

pub trait Tool: Sized {
    fn launch() -> ! {
        match Self::new() {
            Ok(tool) => tool.exec(),
            Err(e) => {
                style::display_error(e);
                exit(1);
            }
        }
    }

    fn new() -> Result<Self, failure::Error>;

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    fn command(self) -> Command;

    fn exec(self) -> ! {
        let mut command = self.command();
        let status = command.status();
        match status {
            Ok(status) if status.success() => {
                exit(0);
            }
            Ok(status) => {
                // FIXME: if None, in unix, find out the signal
                exit(status.code().unwrap_or(1));
            }
            Err(err) => {
                style::display_error(err);
                exit(1);
            }
        }
    }
}

pub struct Script(Command);

pub struct Binary(Command);

pub struct Node(Command);

#[cfg(windows)]
impl Tool for Script {
    fn new() -> Result<Self, failure::Error> {
        unimplemented!()
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        // See: https://github.com/rust-lang/rust/issues/42791
        let mut command = Command::new("cmd.exe");
        command.arg("/C");
        command.arg(exe);
        command.args(args);
        command.env("PATH", path_var);
        Script(command)
    }

    fn command(self) -> Command { self.0 }
}

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    // FIXME: at least in unix, use exec instead
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(not(windows))]
impl Tool for Script {
    fn new() -> Result<Self, failure::Error> {
        unimplemented!()
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Script(command_for(exe, args, path_var))
    }

    fn command(self) -> Command { self.0 }
}

impl Tool for Binary {
    fn new() -> Result<Self, failure::Error> {
        unimplemented!()
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Binary(command_for(exe, args, path_var))
    }

    fn command(self) -> Command { self.0 }
}

impl Tool for Node {
    fn new() -> Result<Self, failure::Error> {
        let mut session = Session::new()?;
        let mut args = args_os();
        // FIXME: make an error kind for this case
        let exe = Path::new(&args.next().unwrap()).file_name().unwrap().to_os_string();
        // FIXME: make an error kind for this case
        let version = session.node()?.unwrap();
        let path_var = env::path_for(&version.to_string());
        Ok(Self::from_components(&exe, args, &path_var))
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Node(command_for(exe, args, path_var))
    }

    fn command(self) -> Command { self.0 }
}
