use std::ffi::OsStr;
use std::iter::once;

use super::executor::{Executor, InternalInstallCommand, PackageInstallCommand, UninstallCommand};
use crate::error::Fallible;
use crate::platform::PlatformSpec;
use crate::tool::package::PackageManager;
use crate::tool::Spec;

pub enum CommandArg<'a> {
    GlobalInstall(InstallArgs<'a>),
    GlobalUninstall(UninstallArgs),
    NotGlobal,
}

impl<'a> CommandArg<'a> {
    /// Parse the given set of arguments to see if they correspond to an npm global command
    pub fn for_npm<S>(args: &'a [S]) -> Fallible<Self>
    where
        S: AsRef<OsStr>,
    {
        // npm global installs will always have `-g` or `--global` somewhere in the argument list
        if !args
            .iter()
            .any(|arg| arg.as_ref() == "-g" || arg.as_ref() == "--global")
        {
            return Ok(CommandArg::NotGlobal);
        }

        let mut positionals = args.iter().filter(is_positional).map(AsRef::as_ref);

        // The first positional argument will always be the command, however npm supports
        // multiple aliases for each command:
        //   -   install: `install`, `i`, `add`, `isntall`
        //   - uninstall: `uninstall`, `unlink`, `remove`, `rm`, `r`
        // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
        // Additionally, if we have a global install or uninstall, all of the remaining positional
        // arguments will be the tools to install or uninstall
        match positionals.next() {
            Some(cmd) if cmd == "install" || cmd == "i" || cmd == "add" || cmd == "isntall" => {
                // The common args for an install should be the command combined with any flags
                let mut common_args = vec![cmd];
                common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));

                Ok(CommandArg::GlobalInstall(InstallArgs {
                    manager: PackageManager::Npm,
                    common_args,
                    tools: positionals.collect(),
                }))
            }
            Some(cmd)
                if cmd == "uninstall"
                    || cmd == "unlink"
                    || cmd == "remove"
                    || cmd == "rm"
                    || cmd == "r" =>
            {
                let tools = positionals
                    .map(|arg| Spec::try_from_str(&arg.to_string_lossy()))
                    .collect::<Fallible<Vec<_>>>()?;

                Ok(CommandArg::GlobalUninstall(UninstallArgs { tools }))
            }
            _ => Ok(CommandArg::NotGlobal),
        }
    }

    /// Parse the given set of arguments to see if they correspond to a Yarn global command
    pub fn for_yarn<S>(args: &'a [S]) -> Fallible<Self>
    where
        S: AsRef<OsStr>,
    {
        let mut positionals = args.iter().filter(is_positional).map(AsRef::as_ref);

        // Yarn globals must always start with `global <command>`
        // If we have a global add or remove, then all of the remaining positional arguments will
        // be the tools to install or uninstall
        match (positionals.next(), positionals.next()) {
            (Some(global), Some(add)) if global == "global" && add == "add" => {
                // The common args for an install should be `global add` and any flags
                let mut common_args = vec![global, add];
                common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));

                Ok(CommandArg::GlobalInstall(InstallArgs {
                    manager: PackageManager::Yarn,
                    common_args,
                    tools: positionals.collect(),
                }))
            }
            (Some(global), Some(remove)) if global == "global" && remove == "remove" => {
                let tools = positionals
                    .map(|arg| Spec::try_from_str(&arg.to_string_lossy()))
                    .collect::<Fallible<Vec<_>>>()?;

                Ok(CommandArg::GlobalUninstall(UninstallArgs { tools }))
            }
            _ => Ok(CommandArg::NotGlobal),
        }
    }
}

/// The arguments passed to a global install command
pub struct InstallArgs<'a> {
    /// The package manager being used
    manager: PackageManager,
    /// Common arguments that apply to each tool (e.g. flags)
    common_args: Vec<&'a OsStr>,
    /// The individual tool arguments
    tools: Vec<&'a OsStr>,
}

impl<'a> InstallArgs<'a> {
    /// Convert these global install arguments into an executor for the command
    ///
    /// If there are multiple packages specified to install, then they will be broken out into
    /// individual commands and run separately. That allows us to keep Volta's sandboxing for each
    /// package while still supporting the ability to install multiple packages at once.
    pub fn executor(self, default_platform: &PlatformSpec) -> Fallible<Executor> {
        let mut executors = Vec::with_capacity(self.tools.len());

        for tool in self.tools {
            match Spec::try_from_str(&tool.to_string_lossy())? {
                Spec::Package(name, _) => {
                    let platform = default_platform.as_default();
                    // The args for an individual install command are the common args combined
                    // with the name of the tool.
                    let args = self.common_args.iter().chain(once(&tool));
                    let command = PackageInstallCommand::new(name, args, platform, self.manager)?;
                    executors.push(command.into());
                }
                internal => executors.push(InternalInstallCommand::new(internal).into()),
            }
        }

        Ok(executors.into())
    }
}

/// The list of tools passed to an uninstall command
pub struct UninstallArgs {
    tools: Vec<Spec>,
}

impl UninstallArgs {
    /// Convert the tools into an executor for the uninstall command
    ///
    /// Since the packages are sandboxed, each needs to be uninstalled separately
    pub fn executor(self) -> Executor {
        let mut executors = Vec::with_capacity(self.tools.len());

        for tool in self.tools {
            executors.push(UninstallCommand::new(tool).into());
        }

        executors.into()
    }
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a.starts_with('-'),
        None => false,
    }
}

fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    !is_flag(arg)
}
