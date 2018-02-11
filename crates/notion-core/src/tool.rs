use std::env::{args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
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
                // ISSUE (#36): if None, in unix, find out the signal
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
        // The best way to launch a script in Windows is to use `cmd.exe`
        // as the executable and pass `"/C"` followed by the name of the
        // script and then its arguments. Unfortunately, the docs aren't
        // super clear about this, but see the discussion at:
        //
        //     https://github.com/rust-lang/rust/issues/42791
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

#[derive(Fail, Debug)]
#[fail(display = "Internal error: tool name could not be determined")]
pub struct NoArg0Error;

fn arg0(args: &mut ArgsOs) -> Result<OsString, failure::Error> {
    let opt = args.next()
        .and_then(|arg0| Path::new(&arg0)
            .file_name()
            .map(|file_name| file_name.to_os_string()));
    if let Some(file_name) = opt {
        Ok(file_name)
    } else {
        Err(NoArg0Error.into())
    }
}

#[derive(Fail, Debug)]
#[fail(display = "No Node version selected")]
pub struct NoGlobalError;

impl Tool for Node {
    fn new() -> Result<Self, failure::Error> {
        let mut session = Session::new()?;
        let mut args = args_os();
        let exe = arg0(&mut args)?;
        let version = if let Some(version) = session.node()? {
            version
        } else {
            return Err(NoGlobalError.into());
        };
        let path_var = env::path_for(&version.to_string());
        Ok(Self::from_components(&exe, args, &path_var))
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Node(command_for(exe, args, path_var))
    }

    fn command(self) -> Command { self.0 }
}
