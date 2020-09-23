use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::parser::CommandArg;
use super::{debug_active_image, debug_no_platform};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, Source, System};
use crate::session::Session;

/// Build an `Executor` for Yarn
///
/// If the command is a global add or remove and we have a default platform available, then we will
/// use custom logic to ensure that the package is correctly installed / uninstalled in the Volta
/// directory.
///
/// If the command is _not_ a global add / remove or we don't have a default platform, then
/// we will allow Yarn to execute the command as usual.
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    if let CommandArg::Global(cmd) = CommandArg::for_yarn(args) {
        if let Some(default_platform) = session.default_platform()? {
            return cmd.executor(default_platform);
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

// #[cfg(test)]
// mod tests {
//     use super::super::CommandArg;
//     use super::check_yarn_add;
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
//     fn handles_global_adds() {
//         match check_yarn_add(&arg_list(&["global", "add", "typescript"])) {
//             CommandArg::GlobalAdd(_) => (),
//             _ => panic!("Doesn't handle global add"),
//         };

//         match check_yarn_add(&arg_list(&["add", "global", "typescript"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle wrong order"),
//         };
//     }

//     #[test]
//     fn handles_global_removes() {
//         match check_yarn_add(&arg_list(&["global", "remove", "typescript"])) {
//             CommandArg::GlobalRemove(_) => (),
//             _ => panic!("Doesn't handle global remove"),
//         };

//         match check_yarn_add(&arg_list(&["remove", "global", "typescript"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle wrong order"),
//         };
//     }

//     #[test]
//     fn ignores_interspersed_flags() {
//         match check_yarn_add(&arg_list(&[
//             "--no-update-notifier",
//             "global",
//             "--no-audit",
//             "add",
//             "--fake-flag",
//             "cowsay",
//         ])) {
//             CommandArg::GlobalAdd(Spec::Package(name, _)) if name == "cowsay" => (),
//             _ => panic!("Doesn't handle flags correctly"),
//         };
//     }

//     #[test]
//     fn treats_invalid_package_as_not_global() {
//         match check_yarn_add(&arg_list(&["global", "add", "//invalid//"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle invalid packages (add)"),
//         };

//         match check_yarn_add(&arg_list(&["global", "remove", "//invalid//"])) {
//             CommandArg::NotGlobal => (),
//             _ => panic!("Doesn't handle invalid packages (remove)"),
//         };
//     }
// }
