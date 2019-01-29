use std::env::ArgsOs;
use std::ffi::OsStr;
use std::process::Command;

use super::{command_for, Tool};
use session::Session;

use notion_fail::{ExitCode, Fallible, NotionFail};

/// Represents a delegated script.
pub struct Script(Command);

#[cfg(windows)]
impl Tool for Script {
    fn new(_session: &mut Session) -> Fallible<Self> {
        throw!(ToolUnimplementedError::new())
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

    fn command(self) -> Command {
        self.0
    }
}

#[cfg(unix)]
impl Tool for Script {
    fn new(_session: &mut Session) -> Fallible<Self> {
        throw!(ToolUnimplementedError::new())
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Script(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "this tool is not yet implemented")]
#[notion_fail(code = "ExecutableNotFound")]
pub(crate) struct ToolUnimplementedError;

impl ToolUnimplementedError {
    pub(crate) fn new() -> Self {
        ToolUnimplementedError
    }
}
