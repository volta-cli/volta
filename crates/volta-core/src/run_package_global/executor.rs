use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

use crate::command::create_command;
use crate::error::{Context, ErrorKind, Fallible};
use crate::platform::{CliPlatform, Platform, System};
use crate::session::Session;
use crate::signal::pass_control_to_shim;
use crate::style::note_prefix;
use crate::tool::package::{DirectInstall, PackageManager};
use crate::tool::Spec;
use log::info;

pub enum Executor {
    Tool(Box<ToolCommand>),
    PackageInstall(Box<PackageInstallCommand>),
    InternalInstall(Box<InternalInstallCommand>),
    Uninstall(Box<UninstallCommand>),
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
            // Internal installs use Volta's logic and don't rely on the environment variables
            Executor::InternalInstall(_) => {}
            // Uninstalls use Volta's logic and don't rely on environment variables
            Executor::Uninstall(_) => {}
        }
    }

    pub fn cli_platform(&mut self, cli: CliPlatform) {
        match self {
            Executor::Tool(cmd) => cmd.cli_platform(cli),
            Executor::PackageInstall(cmd) => cmd.cli_platform(cli),
            // Internal installs use Volta's logic and don't rely on the Node platform
            Executor::InternalInstall(_) => {}
            // Uninstall use Volta's logic and don't rely on the Node platform
            Executor::Uninstall(_) => {}
        }
    }

    pub fn execute(self, session: &mut Session) -> Fallible<ExitStatus> {
        match self {
            Executor::Tool(cmd) => cmd.execute(session),
            Executor::PackageInstall(cmd) => cmd.execute(session),
            Executor::InternalInstall(cmd) => cmd.execute(session),
            Executor::Uninstall(cmd) => cmd.execute(),
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
        Executor::Tool(Box::new(cmd))
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
    platform: Platform,
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
            platform,
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
        self.platform = cli.merge(self.platform.clone());
    }

    /// Runs the install command, applying the necessary modifications to install into the Volta
    /// data directory
    pub fn execute(mut self, session: &mut Session) -> Fallible<ExitStatus> {
        let image = self.platform.checkout(session)?;
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
        Executor::PackageInstall(Box::new(cmd))
    }
}

/// Executor for running an internal install (installing Node, npm, or Yarn using the `volta
/// install` logic)
///
/// Note: This is not intended to be used for Package installs. Those should go through the
/// `PackageInstallCommand` above, to more seamlessly integrate with the package manager
pub struct InternalInstallCommand {
    tool: Spec,
}

impl InternalInstallCommand {
    pub fn new(tool: Spec) -> Self {
        InternalInstallCommand { tool }
    }

    /// Runs the install, using Volta's internal install logic for the appropriate tool
    fn execute(self, session: &mut Session) -> Fallible<ExitStatus> {
        info!(
            "{} using Volta to install {}",
            note_prefix(),
            self.tool.name()
        );

        self.tool.resolve(session)?.install(session)?;

        Ok(ExitStatus::from_raw(0))
    }
}

impl From<InternalInstallCommand> for Executor {
    fn from(cmd: InternalInstallCommand) -> Self {
        Executor::InternalInstall(Box::new(cmd))
    }
}

/// Executor for running a tool uninstall command.
///
/// This will use the `volta uninstall` logic to correctly ensure that the package is fully
/// uninstalled
pub struct UninstallCommand {
    tool: Spec,
}

impl UninstallCommand {
    pub fn new(tool: Spec) -> Self {
        UninstallCommand { tool }
    }

    /// Runs the uninstall with Volta's internal uninstall logic
    fn execute(self) -> Fallible<ExitStatus> {
        info!(
            "{} using Volta to uninstall {}",
            note_prefix(),
            self.tool.name()
        );

        self.tool.uninstall()?;

        Ok(ExitStatus::from_raw(0))
    }
}

impl From<UninstallCommand> for Executor {
    fn from(cmd: UninstallCommand) -> Self {
        Executor::Uninstall(Box::new(cmd))
    }
}
