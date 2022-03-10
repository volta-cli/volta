use std::env;
use std::ffi::OsString;
use std::fs::File;

use super::executor::{Executor, ToolCommand, ToolKind, UninstallCommand};
use super::parser::{CommandArg, InterceptedCommand};
use super::{debug_active_image, debug_no_platform, RECURSION_ENV_VAR};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, System};
use crate::session::{ActivityKind, Session};
use crate::tool::{PackageManifest, Spec};
use crate::version::VersionSpec;

/// Build an `Executor` for npm
///
/// If the command is a global install or uninstall and we have a default platform available, then
/// we will use custom logic to ensure that the package is correctly installed / uninstalled in the
/// Volta directory.
///
/// If the command is _not_ a global install / uninstall or we don't have a default platform, then
/// we will allow npm to execute the command as usual.
pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Npm);
    // Don't re-evaluate the context or global install interception if this is a recursive call
    let platform = match env::var_os(RECURSION_ENV_VAR) {
        Some(_) => None,
        None => {
            match CommandArg::for_npm(args) {
                CommandArg::Global(cmd) => {
                    // For globals, only intercept if the default platform exists
                    if let Some(default_platform) = session.default_platform()? {
                        return cmd.executor(default_platform);
                    }
                }
                CommandArg::Intercepted(InterceptedCommand::Link(link)) => {
                    // For link commands, only intercept if a platform exists
                    if let Some(platform) = Platform::current(session)? {
                        return link.executor(platform, current_project_name(session));
                    }
                }
                CommandArg::Intercepted(InterceptedCommand::Unlink) => {
                    // For unlink, attempt to find the current project name. If successful, treat
                    // this as a global uninstall of the current project.
                    if let Some(name) = current_project_name(session) {
                        // Same as for link, only intercept if a platform exists
                        if Platform::current(session)?.is_some() {
                            return Ok(UninstallCommand::new(Spec::Package(
                                name,
                                VersionSpec::None,
                            ))
                            .into());
                        }
                    }
                }
                _ => {}
            }

            Platform::current(session)?
        }
    };

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

/// Determine the name of the current project, if possible
fn current_project_name(session: &mut Session) -> Option<String> {
    let project = session.project().ok()??;
    let manifest_file = File::open(project.manifest_file()).ok()?;
    let manifest: PackageManifest = serde_json::de::from_reader(manifest_file).ok()?;

    Some(manifest.name)
}
