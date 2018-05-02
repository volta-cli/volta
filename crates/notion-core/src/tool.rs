//! Traits and types for executing command-line tools.

use std::env::{args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::process::{exit, Command};
use std::path::Path;
use std::marker::Sized;

use session::Session;
use notion_fail::{FailExt, Fallible, NotionFail};
use env;
use style;

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    fn launch() -> ! {
        match Self::new() {
            Ok(tool) => tool.exec(),
            Err(e) => {
                style::display_error(&e);
                exit(1);
            }
        }
    }

    /// Constructs a new instance.
    fn new() -> Fallible<Self>;

    /// Constructs a new instance, using the specified command-line and `PATH` variable.
    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    /// Extracts the `Command` from this tool.
    fn command(self) -> Command;

    /// Delegates the current process to this tool.
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
                style::display_error(&err);
                exit(1);
            }
        }
    }
}

/// Represents a delegated script.
pub struct Script(Command);

/// Represents a delegated binary executable.
pub struct Binary(Command);

/// Represents a Node executable.
pub struct Node(Command);

#[cfg(windows)]
impl Tool for Script {
    fn new() -> Fallible<Self> {
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

    fn command(self) -> Command {
        self.0
    }
}

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(unix)]
impl Tool for Script {
    fn new() -> Fallible<Self> {
        unimplemented!()
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Script(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

impl Tool for Binary {
    fn new() -> Fallible<Self> {
        unimplemented!()
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Binary(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Tool name could not be determined")]
struct NoArg0Error;

fn arg0(args: &mut ArgsOs) -> Fallible<OsString> {
    let opt = args.next().and_then(|arg0| {
        Path::new(&arg0)
            .file_name()
            .map(|file_name| file_name.to_os_string())
    });
    if let Some(file_name) = opt {
        Ok(file_name)
    } else {
        Err(NoArg0Error.unknown())
    }
}

#[derive(Fail, Debug)]
#[fail(display = "No Node version selected")]
struct NoGlobalError;

impl NotionFail for NoGlobalError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> i32 {
        2
    }
}

impl Tool for Node {
    fn new() -> Fallible<Self> {
        let mut session = Session::new()?;
        let mut args = args_os();
        let exe = arg0(&mut args)?;
        let version = if let Some(version) = session.current_node()? {
            version
        } else {
            throw!(NoGlobalError.unknown());
        };
        let path_var = env::path_for(&version.to_string());
        Ok(Self::from_components(&exe, args, &path_var))
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Node(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}
