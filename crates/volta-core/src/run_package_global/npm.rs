use std::ffi::OsString;

use super::executor::{Executor, PackageInstallCommand, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform, CommandArg};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;
use crate::tool::package::PackageManager;

/// Build an `Executor` for npm
///
/// If the command is a global install _and_ we have a default platform available, then we will use
/// the `volta install` logic to manage the install and create a shim for any binaries
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    if let CommandArg::GlobalAdd(package) = check_npm_install(args) {
        if let Some(default_platform) = session.default_platform()? {
            let platform = default_platform.as_default();
            let name = package.to_string_lossy().to_string();

            let command = PackageInstallCommand::new(name, args, platform, PackageManager::Npm)?;
            return Ok(command.into());
        }
    }

    let platform = Platform::current(session)?;

    Ok(ToolCommand::new("npm", args, platform, ToolKind::Npm).into())
}

/// Determine the execution context (PATH and failure error message) for npm
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            let image = plat.checkout(session)?;
            let path = image.path()?;
            debug_active_image(&image);

            Ok((path, ErrorKind::BinaryExecError))
        }
        None => {
            let path = System::path()?;
            debug_no_platform();
            Ok((path, ErrorKind::NoPlatform))
        }
    }
}

fn check_npm_install(args: &[OsString]) -> CommandArg<'_> {
    // npm global installs will have `-g` or `--global` somewhere in the argument list
    if !args.iter().any(|arg| arg == "-g" || arg == "--global") {
        return CommandArg::NotGlobalAdd;
    }

    // Filter the set of args to exclude any CLI flags. The first entry will be the npm command
    // followed by any positional parameters
    let mut filtered = args.iter().filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });

    // npm has aliases for "install" as a command: `i`, `install`, `add`, or `isntall`
    // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
    // Additionally, it is only a valid global install if there is a package to install
    match (filtered.next(), filtered.next()) {
        (Some(cmd), Some(package))
            if cmd == "install" || cmd == "i" || cmd == "add" || cmd == "isntall" =>
        {
            CommandArg::GlobalAdd(package.as_os_str())
        }
        _ => CommandArg::NotGlobalAdd,
    }
}
