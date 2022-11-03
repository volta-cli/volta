use std::env;
use std::ffi::OsString;

use super::executor::{Executor, ToolCommand, ToolKind};
use super::{debug_active_image, debug_no_platform, RECURSION_ENV_VAR};
use crate::error::{ErrorKind, Fallible};
use crate::platform::{Platform, Source, System};
use crate::session::{ActivityKind, Session};

pub(super) fn command(args: &[OsString], session: &mut Session) -> Fallible<Executor> {
    session.add_event_start(ActivityKind::Pnpm);
    // Don't re-evaluate the context or global install interception if this is a recursive call
    let platform = match env::var_os(RECURSION_ENV_VAR) {
        Some(_) => None,
        None => {
            // FIXME: Figure out how to intercept pnpm global commands properly.
            // This guard prevents all global commands from running, it should
            // be removed when we fully implement global command interception.
            let is_global = args.iter().any(|f| f == "--global" || f == "-g");
            if is_global {
                return Err(ErrorKind::Unimplemented {
                    feature: "pnpm global commands".into(),
                }
                .into());
            }

            Platform::current(session)?
        }
    };

    Ok(ToolCommand::new("pnpm", args, platform, ToolKind::Pnpm).into())
}

/// Determine the execution context (PATH and failure error message) for pnpm
pub(super) fn execution_context(
    platform: Option<Platform>,
    session: &mut Session,
) -> Fallible<(OsString, ErrorKind)> {
    match platform {
        Some(plat) => {
            validate_platform_pnpm(&plat)?;

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

fn validate_platform_pnpm(platform: &Platform) -> Fallible<()> {
    match &platform.pnpm {
        Some(_) => Ok(()),
        None => match platform.node.source {
            Source::Project => Err(ErrorKind::NoProjectPnpm.into()),
            Source::Default | Source::Binary => Err(ErrorKind::NoDefaultPnpm.into()),
            Source::CommandLine => Err(ErrorKind::NoCommandLinePnpm.into()),
        },
    }
}
