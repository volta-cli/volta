use std::env::args_os;
use std::ffi::{OsStr, OsString};
use std::rc::Rc;

use super::{debug_tool_message, intercept_global_installs, CommandArg, ToolCommand};
use crate::error::ErrorDetails;
use crate::platform::{PlatformSpec, Source};
use crate::session::{ActivityKind, Session};

use log::debug;
use volta_fail::{throw, Fallible};

pub(crate) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Yarn);

    match get_yarn_platform(session)? {
        Some(platform) => {
            if intercept_global_installs() {
                if let CommandArg::GlobalAdd(package) = check_yarn_add() {
                    throw!(ErrorDetails::NoGlobalInstalls { package });
                }
            }

            // Note: If we've gotten this far, we know there is a yarn version set
            debug_tool_message("yarn", platform.yarn().unwrap());

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("yarn"), args, &path))
        }
        None => {
            debug!("Could not find Volta-managed yarn, delegating to system");
            ToolCommand::passthrough(OsStr::new("yarn"), args, ErrorDetails::NoPlatform)
        }
    }
}

/// Determine the correct platform (project or default) and check if yarn is set for that platform
fn get_yarn_platform(session: &mut Session) -> Fallible<Option<Rc<dyn PlatformSpec>>> {
    match session.current_platform()? {
        Some(platform) => match platform.yarn() {
            Some(_) => Ok(Some(platform)),
            None => match platform.node().source {
                Source::Project => Err(ErrorDetails::NoProjectYarn.into()),
                Source::Default | Source::Binary => Err(ErrorDetails::NoDefaultYarn.into()),
            },
        },
        None => Ok(None),
    }
}

fn check_yarn_add() -> CommandArg {
    // Yarn global installs must be of the form `yarn global add`
    // However, they may have options intermixed, e.g. yarn --verbose global add ember-cli
    let mut args = args_os().skip(1).filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });

    if (args.next(), args.next()) == (Some(OsString::from("global")), Some(OsString::from("add"))) {
        CommandArg::GlobalAdd(args.next())
    } else {
        CommandArg::NotGlobalAdd
    }
}
