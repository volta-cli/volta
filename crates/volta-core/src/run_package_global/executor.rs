use std::ffi::OsStr;
use std::process::{Command, ExitStatus};

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::{CliPlatform, Platform, System};
use crate::session::Session;
use crate::signal::pass_control_to_shim;
use crate::tool::package::{DirectInstall, PackageManager};

pub enum Executor {
    Tool(ToolCommand),
    PackageInstall(PackageInstallCommand),
}

impl Executor {
    pub fn envs<E, K, V>(&mut self, envs: E)
    where
        E: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        match self {
            Executor::Tool(cmd) => cmd.envs(envs),
            Executor::PackageInstall(cmd) => cmd.envs(envs),
        }
    }

    pub fn cli_platform(&mut self, cli: CliPlatform) {
        match self {
            Executor::Tool(cmd) => cmd.cli_platform(cli),
            Executor::PackageInstall(cmd) => cmd.cli_platform(cli),
        }
    }

    pub fn execute(self, session: &mut Session) -> Fallible<ExitStatus> {
        match self {
            Executor::Tool(cmd) => cmd.execute(session),
            Executor::PackageInstall(cmd) => cmd.execute(session),
        }
    }
}

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

impl From<ToolCommand> for Executor {
    fn from(cmd: ToolCommand) -> Self {
        Executor::Tool(cmd)
    }
}

/// Process builder for launching a package install command (e.g. `npm install --global`)
///
/// This will use a `DirectInstall` instance to modify the command before running to point it to
/// the Volta directory. It will also complete the install, writing config files and shims
pub struct PackageInstallCommand {
    /// The command that will ultimately be executed
    command: Command,
    /// The installer that modifies the command as necessary and provides the completion method
    installer: DirectInstall,
    /// The platform to use when running the command.
    ///
    /// Note: This will always be set to `Some`, it being an `Option` is an implementation detail
    /// to allow the `cli_platform` method to move the Platform out when merging with CliPlatform
    platform: Option<Platform>,
}

impl PackageInstallCommand {
    pub fn new<A, S>(
        name: String,
        args: A,
        platform: Platform,
        manager: PackageManager,
    ) -> Fallible<Self>
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let installer = DirectInstall::new(name, manager)?;

        let mut command = match manager {
            PackageManager::Npm => create_command("npm"),
            PackageManager::Yarn => create_command("yarn"),
        };
        command.args(args);

        Ok(PackageInstallCommand {
            command,
            installer,
            platform: Some(platform),
        })
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
        // Invariant: Platform is always set, except during the execution of `CliPlatform::merge`
        // `CliPlatform::merge` _can't_ call methods on this object, unwrapping is safe
        self.platform = Some(cli.merge(self.platform.take().unwrap()));
    }

    /// Runs the install command, applying the necessary modifications to install into the Volta
    /// data directory
    pub fn execute(mut self, session: &mut Session) -> Fallible<ExitStatus> {
        // Invariant: Platform is always set except during the execution of `cli_platform`
        // Since that function will not call this one, it is safe to unwrap.
        let image = self.platform.unwrap().checkout(session)?;
        let path = image.path()?;

        self.command.env("PATH", path);
        self.installer.setup_command(&mut self.command);

        let status = self
            .command
            .status()
            .with_context(|| ErrorKind::BinaryExecError)?;

        if status.success() {
            self.installer.complete_install(&image)?;
        }

        Ok(status)
    }
}

impl From<PackageInstallCommand> for Executor {
    fn from(cmd: PackageInstallCommand) -> Self {
        Executor::PackageInstall(cmd)
    }
}
