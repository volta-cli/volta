//! Traits and types for executing command-line tools.

use std::env::{args_os, ArgsOs};
use std::ffi::{OsStr, OsString};
use std::io;
use std::marker::Sized;
use std::path::Path;
use std::process::{Command, ExitStatus};

use notion_fail::{ExitCode, FailExt, Fallible, NotionError, NotionFail};
use path;
use session::{ActivityKind, Session};
use style;
use version::VersionSpec;

fn display_error(err: &NotionError) {
    if err.is_user_friendly() {
        style::display_error(style::ErrorContext::Shim, err);
    } else {
        style::display_unknown_error(style::ErrorContext::Shim, err);
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

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "this tool is not yet implemented")]
#[notion_fail(code = "ExecutableNotFound")]
pub(crate) struct ToolUnimplementedError;

impl ToolUnimplementedError {
    pub(crate) fn new() -> Self {
        ToolUnimplementedError
    }
}

/// Represents a command-line tool that Notion shims delegate to.
pub trait Tool: Sized {
    fn launch() -> ! {
        let mut session = match Session::new() {
            Ok(session) => session,
            Err(err) => {
                display_error(&err);
                ExitCode::ExecutionFailure.exit();
            }
        };

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

/// Represents a delegated script.
pub struct Script(Command);

/// Represents a delegated binary executable.
pub struct Binary(Command);

/// Represents a Node executable.
pub struct Node(Command);

/// Represents a `npm` executable.
pub struct Npm(Command);

/// Represents a `npx` executable.
pub struct Npx(Command);

/// Represents a Yarn executable.
pub struct Yarn(Command);

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

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
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
#[fail(display = "No toolchain available to run shim {}", shim_name)]
#[notion_fail(code = "ExecutionFailure")]
pub(crate) struct NoToolChainError {
    shim_name: String,
}

impl NoToolChainError {
    pub(crate) fn for_shim(shim_name: String) -> NoToolChainError {
        NoToolChainError { shim_name }
    }
}

impl Tool for Binary {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Binary);

        let mut args = args_os();
        let exe = arg0(&mut args)?;

        // first try to use the project toolchain
        if let Some(project) = session.project() {
            // check if the executable is a direct dependency
            if project.has_direct_bin(&exe)? {
                // use the full path to the file
                let mut path_to_bin = project.local_bin_dir();
                path_to_bin.push(&exe);

                // if we're in a pinned project, use the project's platform.
                if let Some(ref platform) = session.project_platform() {
                    return Ok(Self::from_components(
                        &path_to_bin.as_os_str(),
                        args,
                        &platform.path()?,
                    ));
                }

                // otherwise use the user platform.
                if let Some(ref platform) = session.user_platform()? {
                    return Ok(Self::from_components(
                        &path_to_bin.as_os_str(),
                        args,
                        &platform.path()?,
                    ))
                }

                // if there's no user platform selected, fail.
                throw!(NoSuchToolError {
                    tool: "Node".to_string()
                });
            }
        }

        // next try to use the user toolchain
        if let Some(ref platform) = session.user_platform()? {
            // use the full path to the binary
            // ISSUE (#160): Look up the platform image bound to the user tool.
            let node_str = platform.node.runtime.to_string();
            let npm_str = platform.node.npm.to_string();
            let mut third_p_bin_dir = path::node_image_3p_bin_dir(&node_str, &npm_str)?;
            third_p_bin_dir.push(&exe);
            return Ok(Self::from_components(
                &third_p_bin_dir.as_os_str(),
                args,
                &platform.path()?,
            ));
        };

        // at this point, there is no project or user toolchain
        // the user is executing a Notion shim that doesn't have a way to execute it
        throw!(NoToolChainError::for_shim(
            exe.to_string_lossy().to_string()
        ));
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

#[derive(Debug, Fail, NotionFail)]
#[fail(display = r#"
No {} version selected.

See `notion help use` for help adding {} to a project toolchain.

See `notion help install` for help adding {} to your personal toolchain."#, tool, tool, tool)]
#[notion_fail(code = "NoVersionMatch")]
struct NoSuchToolError {
    tool: String,
}

impl Tool for Node {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Node);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            session.prepare_image(platform)?;
            Ok(Self::from_components(&exe, args, &platform.path()?))
        } else {
            throw!(NoSuchToolError {
                tool: "Node".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Node(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

impl Tool for Yarn {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Yarn);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            session.prepare_image(platform)?;
            Ok(Self::from_components(&exe, args, &platform.path()?))
        } else {
            throw!(NoSuchToolError {
                tool: "Yarn".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Yarn(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }

    /// Perform any tasks which must be run after the tool runs but before exiting.
    fn finalize(session: &Session, maybe_status: &io::Result<ExitStatus>) {
        if let Ok(_) = maybe_status {
            if let Some(project) = session.project() {
                let errors = project.autoshim();

                for error in errors {
                    display_error(&error);
                }
            }
        }
    }
}

impl Tool for Npm {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Npm);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            session.prepare_image(platform)?;

            if platform.yarn.is_some() {
                // If Yarn is enabled for the current project, warn the user that they shouldn't be using npm
                style::display_warning(&String::from("This project is configured to use yarn, npm should be avoided"));
            }
            Ok(Self::from_components(&exe, args, &platform.path()?))
        } else {
            // Using 'Node' as the tool name since the npm version is derived from the Node version
            // This way the error message will prompt the user to add 'Node' to their toolchain, instead of 'npm'
            throw!(NoSuchToolError {
                tool: "Node".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Npm(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }

    /// Perform any tasks which must be run after the tool runs but before exiting
    fn finalize(session: &Session, maybe_status: &io::Result<ExitStatus>) {
        if let Ok(_) = maybe_status {
            if let Some(project) = session.project() {
                let errors = project.autoshim();

                for error in errors {
                    display_error(&error);
                }
            }
        }
    }
}

#[derive(Debug, Fail, NotionFail)]
#[fail(display = r#"
'npx' is only available on Node >= 8.2.0

This project is configured to use version {} of Node."#, version)]
#[notion_fail(code = "ExecutableNotFound")]
struct NpxNotAvailableError {
    version: String,
}

impl Tool for Npx {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Npx);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            session.prepare_image(platform)?;

            // npx was only included with Node >= 8.2.0. If less than that, we should include a helpful error message
            let required_node = VersionSpec::parse_requirements(">= 8.2.0")?;
            if required_node.matches(&platform.node) {
                Ok(Self::from_components(&exe, args, &platform.path()?))
            } else {
                throw!(NpxNotAvailableError {
                    version: platform.node_str.clone()
                });
            }
        } else {
            // Using 'Node' as the tool name since the npx version is derived from the Node version
            // This way the error message will prompt the user to add 'Node' to their toolchain, instead of 'npx'
            throw!(NoSuchToolError {
                tool: "Node".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Npx(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}
