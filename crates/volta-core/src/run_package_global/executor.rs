use std::ffi::OsStr;
use std::process::{Command, ExitStatus};

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::{CliPlatform, Platform, System};
use crate::session::Session;
use crate::signal::pass_control_to_shim;

/// Process builder for launching a Volta-managed tool
///
/// Tracks the Platform as well as what kind of tool is being executed, to allow individual tools
/// to customize the behavior before execution.
pub struct ToolCommand {
    command: Command,
    platform: Option<Platform>,
    kind: ToolKind,
}

/// The kind of tool being executed, used to determine the correct execution context
pub enum ToolKind {
    Node,
    Npm,
    Npx,
    Yarn,
    ProjectLocalBinary(String),
    DefaultBinary(String),
    Bypass(String),
}

impl ToolCommand {
    pub fn new<E, A, S>(exe: E, args: A, platform: Option<Platform>, kind: ToolKind) -> Self
    where
        E: AsRef<OsStr>,
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = create_command(exe);
        command.args(args);

        Self {
            command,
            platform,
            kind,
        }
    }

    /// Adds or updates environment variables that the command will use
    pub fn envs<E, K, V>(&mut self, envs: E)
    where
        E: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(envs);
    }

    /// Updates the Platform for the command to include values from the command-line
    pub fn cli_platform(&mut self, cli: CliPlatform) {
        self.platform = match self.platform.take() {
            Some(base) => Some(cli.merge(base)),
            None => cli.into(),
        };
    }

    /// Runs the command, returning the `ExitStatus` if it successfully launches
    pub fn execute(mut self, session: &mut Session) -> Fallible<ExitStatus> {
        let (path, on_failure) = match self.kind {
            ToolKind::Node => super::node::execution_context(self.platform, session)?,
            ToolKind::Npm => super::npm::execution_context(self.platform, session)?,
            ToolKind::Npx => super::npx::execution_context(self.platform, session)?,
            ToolKind::Yarn => super::yarn::execution_context(self.platform, session)?,
            ToolKind::DefaultBinary(bin) => {
                super::binary::default_execution_context(bin, self.platform, session)?
            }
            ToolKind::ProjectLocalBinary(bin) => {
                super::binary::local_execution_context(bin, self.platform, session)?
            }
            ToolKind::Bypass(command) => (System::path()?, ErrorKind::BypassError { command }),
        };

        self.command.env("PATH", path);

        pass_control_to_shim();
        self.command.status().with_context(|| on_failure)
    }
}
