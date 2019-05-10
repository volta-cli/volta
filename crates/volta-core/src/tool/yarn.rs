use std::env::args_os;
use std::ffi::{OsStr, OsString};
use std::rc::Rc;

use super::{intercept_global_installs, CommandArg, ToolCommand};
use crate::error::ErrorDetails;
use crate::platform::PlatformSpec;
use crate::session::{ActivityKind, Session};

use volta_fail::{throw, Fallible};

pub(super) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Yarn);

    match get_yarn_platform(session)? {
        Some(ref platform) => {
            if intercept_global_installs() {
                if let CommandArg::GlobalAdd(package) = check_yarn_add() {
                    throw!(ErrorDetails::NoGlobalInstalls { package });
                }
            }

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("yarn"), args, &path))
        }
        None => ToolCommand::passthrough(OsStr::new("yarn"), args, ErrorDetails::NoPlatform),
    }
}

/// Determine the correct platform (project or user) and check if yarn is set for that platform
fn get_yarn_platform(session: &mut Session) -> Fallible<Option<Rc<PlatformSpec>>> {
    // First check if we are in a pinned project
    if let Some(platform) = session.project_platform()? {
        return match platform.yarn {
            Some(_) => Ok(Some(platform)),
            None => Err(ErrorDetails::NoProjectYarn.into()),
        };
    }

    // If not, fall back to the user platform
    if let Some(platform) = session.user_platform()? {
        return match platform.yarn {
            Some(_) => Ok(Some(platform)),
            None => Err(ErrorDetails::NoUserYarn.into()),
        };
    }

    Ok(None)
}

fn check_yarn_add() -> CommandArg {
    // Yarn global installs must be of the form `yarn global add`
    // However, they may have options intermixed, e.g. yarn --verbose global add ember-cli
    let mut args = args_os().skip(1).filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with("-"),
        None => true,
    });

    if (args.next(), args.next()) == (Some(OsString::from("global")), Some(OsString::from("add"))) {
        CommandArg::GlobalAdd(args.next())
    } else {
        CommandArg::NotGlobalAdd
    }
}
