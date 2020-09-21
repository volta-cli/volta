use std::ffi::OsString;

use super::executor::{
    Executor, InternalInstallCommand, PackageInstallCommand, ToolCommand, ToolKind,
    UninstallCommand,
};
use super::{debug_active_image, debug_no_platform, CommandArg};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;
use crate::tool::package::PackageManager;
use crate::tool::Spec;

/// Build an `Executor` for npm
///
/// - If the command is a global package install _and_ we have a default platform available, then
///   we will install the package into the Volta data directory and generate appropriate shims.
/// - If the command is a global install of a Volta-managed tool (Node, npm, Yarn), then we will
///   use Volta's internal install logic.
/// - Otherwise, we allow npm to execute the command as usual
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    match check_npm_install(args) {
        CommandArg::GlobalAdd(Spec::Package(name, _)) => {
            if let Some(default_platform) = session.default_platform()? {
                let platform = default_platform.as_default();
                let command =
                    PackageInstallCommand::new(name, args, platform, PackageManager::Npm)?;
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

/// Using the provided arguments, check if the command is a valid global install or uninstall
///
/// Note: We treat the case of an invalid package name as _not_ a global install,
/// to allow npm to show the appropriate error message.
fn check_npm_install(args: &[OsString]) -> CommandArg {
    // npm global installs will have `-g` or `--global` somewhere in the argument list
    if !args.iter().any(|arg| arg == "-g" || arg == "--global") {
        return CommandArg::NotGlobal;
    }

    // Filter the set of args to exclude any CLI flags. The first entry will be the npm command
    // followed by any positional parameters
    let mut filtered = args.iter().filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });

    // npm has aliases for "install" as a command: `i`, `install`, `add`, or `isntall`
    // aliases for "uninstall" as a command: `unlink`, `remove`, `rm`, `r`
    // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
    // Additionally, it is only a valid global install/uninstall if there is a valid package
    match (filtered.next(), filtered.next()) {
        (Some(cmd), Some(package))
            if cmd == "install" || cmd == "i" || cmd == "add" || cmd == "isntall" =>
        {
            match Spec::try_from_str(&package.to_string_lossy()) {
                Ok(tool) => CommandArg::GlobalAdd(tool),
                Err(_) => CommandArg::NotGlobal,
            }
        }
        (Some(cmd), Some(package))
            if cmd == "uninstall"
                || cmd == "unlink"
                || cmd == "remove"
                || cmd == "rm"
                || cmd == "r" =>
        {
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
    use super::check_npm_install;
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
    fn handles_global_flags() {
        match check_npm_install(&arg_list(&["install", "-g", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle short form (-g)"),
        };

        match check_npm_install(&arg_list(&["install", "--global", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle long form"),
        };

        match check_npm_install(&arg_list(&["install", "typescript"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle non-globals"),
        };
    }

    #[test]
    fn handles_install_aliases() {
        match check_npm_install(&arg_list(&["install", "-g", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle install"),
        };

        match check_npm_install(&arg_list(&["i", "-g", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle short form (i)"),
        };

        match check_npm_install(&arg_list(&["add", "-g", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle add"),
        };

        match check_npm_install(&arg_list(&["isntall", "-g", "typescript"])) {
            CommandArg::GlobalAdd(_) => (),
            _ => panic!("Doesn't handle misspelling"),
        };
    }

    #[test]
    fn handles_uninstall_aliases() {
        match check_npm_install(&arg_list(&["uninstall", "-g", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle uninstall"),
        };

        match check_npm_install(&arg_list(&["unlink", "-g", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle unlink"),
        };

        match check_npm_install(&arg_list(&["remove", "-g", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle remove"),
        };

        match check_npm_install(&arg_list(&["rm", "-g", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle short form (rm)"),
        };

        match check_npm_install(&arg_list(&["r", "-g", "typescript"])) {
            CommandArg::GlobalRemove(_) => (),
            _ => panic!("Doesn't handle short form (r)"),
        };
    }

    #[test]
    fn ignores_interspersed_flags() {
        match check_npm_install(&arg_list(&[
            "--no-update-notifier",
            "install",
            "--no-audit",
            "--global",
            "cowsay",
        ])) {
            CommandArg::GlobalAdd(Spec::Package(name, _)) if name == "cowsay" => (),
            _ => panic!("Doesn't handle flags correctly"),
        };
    }

    #[test]
    fn treats_invalid_package_as_not_global() {
        match check_npm_install(&arg_list(&["install", "-g", "//invalid//"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle invalid packages (install)"),
        };

        match check_npm_install(&arg_list(&["uninstall", "-g", "//invalid//"])) {
            CommandArg::NotGlobal => (),
            _ => panic!("Doesn't handle invalid packages (uninstall)"),
        };
    }
}
