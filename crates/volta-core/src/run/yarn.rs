use std::env::args_os;
use std::ffi::OsStr;

use super::{debug_tool_message, intercept_global_installs, CommandArg, ToolCommand};
use crate::error::ErrorDetails;
use crate::platform::{CliPlatform, Platform, Source};
use crate::session::{ActivityKind, Session};

use log::debug;
use volta_fail::{throw, Fallible};

pub(crate) fn command(cli: CliPlatform, session: &mut Session) -> Fallible<ToolCommand> {
    session.add_event_start(ActivityKind::Yarn);

    match get_yarn_platform(cli, session)? {
        Some(platform) => {
            if intercept_global_installs() {
                if let CommandArg::GlobalAdd(package) = check_yarn_add() {
                    throw!(ErrorDetails::NoGlobalInstalls { package });
                }
            }

            // Note: If we've gotten this far, we know there is a yarn version set
            debug_tool_message("yarn", platform.yarn.as_ref().unwrap());

            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("yarn"), &path))
        }
        None => {
            debug!("Could not find Volta-managed yarn, delegating to system");
            ToolCommand::passthrough(OsStr::new("yarn"), ErrorDetails::NoPlatform)
        }
    }
}

/// Determine the correct platform (project or default) and check if yarn is set for that platform
fn get_yarn_platform(cli: CliPlatform, session: &mut Session) -> Fallible<Option<Platform>> {
    match Platform::with_cli(cli, session)? {
        Some(platform) => match &platform.yarn {
            Some(_) => Ok(Some(platform)),
            None => match platform.node.source {
                Source::Project => Err(ErrorDetails::NoProjectYarn.into()),
                Source::Default | Source::Binary => Err(ErrorDetails::NoDefaultYarn.into()),
                Source::CommandLine => Err(ErrorDetails::NoCommandLineYarn.into()),
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

    match (args.next(), args.next()) {
        (Some(global), Some(add)) if global == "global" && add == "add" => {
            CommandArg::GlobalAdd(args.next())
        }
        _ => CommandArg::NotGlobalAdd,
    }
}
