use std::ffi::OsString;

use super::executor::{Executor, PackageInstallCommand, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform, CommandArg};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, Source, System};
use crate::session::Session;
use crate::tool::package::PackageManager;

/// Build an `Executor` for Yarn
///
/// If the ocmmand is a global add _and_ we have a default platform available, then we will use
/// the `volta install` logic to manage the install and create a shim for any binaries
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    if let CommandArg::GlobalAdd(package) = check_yarn_add(args) {
        if let Some(default_platform) = session.default_platform()? {
            let platform = default_platform.as_default();
            let name = package.to_string_lossy().to_string();

            let command = PackageInstallCommand::new(name, args, platform, PackageManager::Yarn)?;
            return Ok(command.into());
        }
    }

    let platform = Platform::current(session)?;

    Ok(ToolCommand::new("yarn", args, platform, ToolKind::Yarn).into())
}

/// Determine the execution context (PATH and failure error message) for Yarn
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            validate_platform_yarn(&plat)?;

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

fn validate_platform_yarn(platform: &Platform) -> Fallible<()> {
    match &platform.yarn {
        Some(_) => Ok(()),
        None => match platform.node.source {
            Source::Project => Err(ErrorKind::NoProjectYarn.into()),
            Source::Default | Source::Binary => Err(ErrorKind::NoDefaultYarn.into()),
            Source::CommandLine => Err(ErrorKind::NoCommandLineYarn.into()),
        },
    }
}

fn check_yarn_add(args: &[OsString]) -> CommandArg<'_> {
    // Yarn global installs must be of the form `yarn global add <package>`
    // However, they may have options intermixed, e.g. `yarn --verbose global add ember-cli`
    let mut filtered = args.iter().filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });

    match (filtered.next(), filtered.next(), filtered.next()) {
        (Some(global), Some(add), Some(package)) if global == "global" && add == "add" => {
            CommandArg::GlobalAdd(package.as_os_str())
        }
        _ => CommandArg::NotGlobalAdd,
    }
}
