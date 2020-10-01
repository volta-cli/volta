use std::ffi::OsString;

use super::executor::{
    Executor, InternalInstallCommand, PackageInstallCommand, ToolCommand, ToolKind,
    UninstallCommand,
};
use super::{debug_active_image, debug_no_platform, CommandArg};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, Source, System};
use crate::session::Session;
use crate::tool::package::PackageManager;
use crate::tool::Spec;

/// Build an `Executor` for Yarn
///
/// - If the command is a global package install _and_ we have a default platform available, then
///   we will install the package into the Volta data directory and generate appropriate shims.
/// - If the command is a global install of a Volta-managed tool (Node, npm, Yarn), then we will
///   use Volta's internal install logic.
/// - Otherwise, we allow npm to execute the command as usual
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    match check_yarn_add(args) {
        CommandArg::GlobalAdd(Spec::Package(name, _)) => {
            if let Some(default_platform) = session.default_platform()? {
                let platform = default_platform.as_default();
                let command =
                    PackageInstallCommand::new(name, args, platform, PackageManager::Yarn)?;
                return Ok(command.into());
            }
        }
        CommandArg::GlobalAdd(tool) => {
            return Ok(InternalInstallCommand::new(tool).into());
        }
        CommandArg::GlobalRemove(tool) => {
            return Ok(UninstallCommand::new(tool).into());
        }
        _ => {}
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

/// Using the provided arguments, check if the command is a valid global add or remove
///
/// Note: We treat the case of an invalid package name as _not_ a global add, to allow
/// Yarn to show the appropriate error message.
fn check_yarn_add(args: &[OsString]) -> CommandArg {
    // Yarn global installs must be of the form `yarn global add <package>`
    // However, they may have options intermixed, e.g. `yarn --verbose global add ember-cli`
    let mut filtered = args.iter().filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });

    match (filtered.next(), filtered.next(), filtered.next()) {
        (Some(global), Some(add), Some(package)) if global == "global" && add == "add" => {
            match Spec::try_from_str(&package.to_string_lossy()) {
                Ok(tool) => CommandArg::GlobalAdd(tool),
                Err(_) => CommandArg::NotGlobal,
            }
        }
        (Some(global), Some(remove), Some(package)) if global == "global" && remove == "remove" => {
            match Spec::try_from_str(&package.to_string_lossy()) {
                Ok(tool) => CommandArg::GlobalRemove(tool),
                Err(_) => CommandArg::NotGlobal,
            }
        }
        _ => CommandArg::NotGlobal,
    }
}

#[cfg(test)]
mod tests {
    use super::super::CommandArg;
    use super::check_yarn_add;
    use crate::tool::Spec;
    use std::ffi::{OsStr, OsString};

    fn arg_list<A, S>(args: A) -> Vec<OsString>
    where
        A: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        args.into_iter().map(|a| a.as_ref().to_owned()).collect()
    }

    #[test]
    fn handles_global_adds() {
        match check_yarn_add(&arg_list(&["global", "add", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle global add"),
        };

        match check_yarn_add(&arg_list(&["add", "global", "typescript"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle wrong order"),
        };
    }

    #[test]
    fn handles_global_removes() {
        match check_yarn_add(&arg_list(&["global", "remove", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle global remove"),
        };

        match check_yarn_add(&arg_list(&["remove", "global", "typescript"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle wrong order"),
        };
    }

    #[test]
    fn ignores_interspersed_flags() {
        match check_yarn_add(&arg_list(&[
            "--no-update-notifier",
            "global",
            "--no-audit",
            "add",
            "--fake-flag",
            "cowsay",
        ])) {
            CommandArg::GlobalAdd(Spec::Package(name, _)) if name == "cowsay" => (),
            _ => panic!("Doesn't handle flags correctly"),
        };
    }

    #[test]
    fn treats_invalid_package_as_not_global() {
        match check_yarn_add(&arg_list(&["global", "add", "//invalid//"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle invalid packages (add)"),
        };

        match check_yarn_add(&arg_list(&["global", "remove", "//invalid//"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle invalid packages (remove)"),
        };
    }
}
