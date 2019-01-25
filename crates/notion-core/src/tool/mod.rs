//! Traits and types for executing command-line tools.

use std::env::{self, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};

use notion_fail::{ExitCode, FailExt, Fallible, NotionError, NotionFail};
use session::{ActivityKind, Session};
use style;
use version::VersionSpec;

mod binary;
mod node;
mod npm;
mod npx;
mod script;
mod yarn;

pub use self::binary::Binary;
pub use self::node::Node;
pub use self::npm::Npm;
pub use self::npx::Npx;
pub use self::script::Script;
pub use self::yarn::Yarn;

fn display_error(err: &NotionError) {
    if err.is_user_friendly() {
        style::display_error(style::ErrorContext::Shim, err);
    } else {
        style::display_unknown_error(style::ErrorContext::Shim, err);
    }
}

pub enum ToolSpec {
    Node(VersionSpec),
    Yarn(VersionSpec),
    Npm(VersionSpec),
    Package(String, VersionSpec),
}

impl ToolSpec {
    pub fn from_str(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => ToolSpec::Node(version),
            "yarn" => ToolSpec::Yarn(version),
            "npm" => ToolSpec::Npm(version),
            package => ToolSpec::Package(package.to_string(), version),
        }
    }
}

impl Debug for ToolSpec {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &ToolSpec::Node(ref version) => format!("node version {}", version),
            &ToolSpec::Yarn(ref version) => format!("yarn version {}", version),
            &ToolSpec::Npm(ref version) => format!("npm version {}", version),
            &ToolSpec::Package(ref name, ref version) => format!("{} version {}", name, version),
        };
        f.write_str(&s)
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "{}", error)]
#[notion_fail(code = "ExecutionFailure")]
pub(crate) struct BinaryExecError {
    pub(crate) error: String,
}

impl BinaryExecError {
    pub(crate) fn from_io_error(error: &io::Error) -> Self {
        if let Some(inner_err) = error.get_ref() {
            BinaryExecError {
                error: inner_err.to_string(),
            }
        } else {
            BinaryExecError {
                error: error.to_string(),
            }
        }
    }
}

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    fn launch() -> ! {
        let mut session = Session::new();

        session.add_event_start(ActivityKind::Tool);

        match Self::new(&mut session) {
            Ok(tool) => {
                tool.exec(session);
            }
            Err(err) => {
                display_error(&err);
                session.add_event_error(ActivityKind::Tool, &err);
                session.exit(ExitCode::ExecutionFailure);
            }
        }
    }

    /// Constructs a new instance.
    fn new(&mut Session) -> Fallible<Self>;

    /// Constructs a new instance, using the specified command-line and `PATH` variable.
    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    /// Extracts the `Command` from this tool.
    fn command(self) -> Command;

    /// Perform any tasks which must be run after the tool runs but before exiting.
    fn finalize(_session: &Session, _maybe_status: &io::Result<ExitStatus>) {}

    /// Delegates the current process to this tool.
    fn exec(self, mut session: Session) -> ! {
        let mut command = self.command();
        let status = command.status();
        Self::finalize(&session, &status);
        match status {
            Ok(status) if status.success() => {
                session.add_event_end(ActivityKind::Tool, ExitCode::Success);
                session.exit(ExitCode::Success);
            }
            Ok(status) => {
                // ISSUE (#36): if None, in unix, find out the signal
                let code = status.code().unwrap_or(1);
                session.add_event_tool_end(ActivityKind::Tool, code);
                session.exit_tool(code);
            }
            Err(err) => {
                let notion_err = err.with_context(BinaryExecError::from_io_error);
                display_error(&notion_err);
                session.add_event_error(ActivityKind::Tool, &notion_err);
                session.exit(ExitCode::ExecutionFailure);
            }
        }
    }
}

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
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

#[derive(Debug, Fail, NotionFail)]
#[fail(
    display = r#"
No {} version selected.

See `notion help pin` for help adding {} to a project toolchain.

See `notion help install` for help adding {} to your personal toolchain."#,
    tool, tool, tool
)]
#[notion_fail(code = "NoVersionMatch")]
struct NoSuchToolError {
    tool: String,
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = r#"
Global package installs are not recommended.

Consider using `notion install` to add a package to your toolchain (see `notion help install for more info).

Set the NOTION_ALLOW_GLOBAL environment variable to install anyway."#)]
#[notion_fail(code = "InvalidArguments")]
struct NoGlobalInstallError;

fn intercept_global_installs() -> bool {
    // We should only intercept global installs if the NOTION_ALLOW_GLOBAL variable is not set
    env::var_os("NOTION_ALLOW_GLOBAL").is_none()
}
