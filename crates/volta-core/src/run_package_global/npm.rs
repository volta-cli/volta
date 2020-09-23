use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::parser::CommandArg;
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::Session;

/// Build an `Executor` for npm
///
/// If the command is a global install or uninstall and we have a default platform available, then
/// we will use custom logic to ensure that the package is correctly installed / uninstalled in the
/// Volta directory.
///
/// If the command is _not_ a global install / uninstall or we don't have a default platform, then
/// we will allow npm to execute the command as usual.
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    if let CommandArg::Global(cmd) = CommandArg::for_npm(args) {
        if let Some(default_platform) = session.default_platform()? {
            return cmd.executor(default_platform);
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

// #[cfg(test)]
// mod tests {
//     use super::super::CommandArg;
//     use super::check_npm_install;
//     use crate::tool::Spec;
//     use std::ffi::{OsStr, OsString};

//     fn arg_list<A, S>(args: A) -> Vec<OsString>
//     where
//         A: IntoIterator<Item = S>,
//         S: AsRef<OsStr>,
//     {
//         args.into_iter().map(|a| a.as_ref().to_owned()).collect()
//     }

//     #[test]
//     fn handles_global_flags() {
//         match check_npm_install(&arg_list(&["install", "-g", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle short form (-g)"),
//         };

//         match check_npm_install(&arg_list(&["install", "--global", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle long form"),
//         };

//         match check_npm_install(&arg_list(&["install", "typescript"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle non-globals"),
//         };
//     }

//     #[test]
//     fn handles_install_aliases() {
//         match check_npm_install(&arg_list(&["install", "-g", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle install"),
//         };

//         match check_npm_install(&arg_list(&["i", "-g", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle short form (i)"),
//         };

//         match check_npm_install(&arg_list(&["add", "-g", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle add"),
//         };

//         match check_npm_install(&arg_list(&["isntall", "-g", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle misspelling"),
//         };
//     }

//     #[test]
//     fn handles_uninstall_aliases() {
//         match check_npm_install(&arg_list(&["uninstall", "-g", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle uninstall"),
//         };

//         match check_npm_install(&arg_list(&["unlink", "-g", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle unlink"),
//         };

//         match check_npm_install(&arg_list(&["remove", "-g", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle remove"),
//         };

//         match check_npm_install(&arg_list(&["rm", "-g", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle short form (rm)"),
//         };

//         match check_npm_install(&arg_list(&["r", "-g", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle short form (r)"),
//         };
//     }

//     #[test]
//     fn ignores_interspersed_flags() {
//         match check_npm_install(&arg_list(&[
//             "--no-update-notifier",
//             "install",
//             "--no-audit",
//             "--global",
//             "cowsay",
//         ])) {
//             CommandArg::GlobalAdd(Spec::Package(name, _)) if name == "cowsay" => (),
//             _ => panic!("Doesn't handle flags correctly"),
//         };
//     }

//     #[test]
//     fn treats_invalid_package_as_not_global() {
//         match check_npm_install(&arg_list(&["install", "-g", "//invalid//"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle invalid packages (install)"),
//         };

//         match check_npm_install(&arg_list(&["uninstall", "-g", "//invalid//"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle invalid packages (uninstall)"),
//         };
//     }
// }
