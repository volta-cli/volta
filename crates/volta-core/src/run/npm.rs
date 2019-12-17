use std::env::args_os;
use std::ffi::{OsStr, OsString};

use super::{debug_tool_message, intercept_global_installs, CommandArg, ToolCommand};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use log::debug;
use volta_fail::{throw, Fallible};

pub(crate) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Npm);

    match session.current_platform()? {
        Some(platform) => {
            if intercept_global_installs() {
                if let CommandArg::GlobalAdd(package) = check_npm_install() {
                    throw!(ErrorDetails::NoGlobalInstalls { package });
                }
            }
            let image = platform.checkout(session)?;
            let path = image.path()?;

            debug_tool_message("npm", &image.npm);

            Ok(ToolCommand::direct(OsStr::new("npm"), args, &path))
        }
        None => {
            debug!("Could not find Volta-managed npm, delegating to system");
            ToolCommand::passthrough(OsStr::new("npm"), args, ErrorDetails::NoPlatform)
        }
    }
}

fn check_npm_install() -> CommandArg {
    // npm global installs will have `-g` or `--global` somewhere in the
    // argument list
    if !args_os().any(|arg| arg == OsString::from("-g") || arg == OsString::from("--global")) {
        return CommandArg::NotGlobalAdd;
    }

    // Get the same set of args again to iterate over, this time with the
    // command itself skipped and all flags excluded entirely. The first item
    // in that skipped, filter iterator is the command itself.
    let mut args = args_os().skip(1).filter(|arg| match arg.to_str() {
        Some(arg) => !arg.starts_with('-'),
        None => true,
    });
    let command = args.next();

    // They will be specified by the command `i`, `install`, `add` or `isntall`.
    // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
    if command == Some(OsString::from("install"))
        || command == Some(OsString::from("i"))
        || command == Some(OsString::from("isntall"))
        || command == Some(OsString::from("add"))
    {
        // `args` here picks up from where the command lookup left off, so
        // will be the name of the package passed to the command.
        CommandArg::GlobalAdd(args.next())
    } else {
        CommandArg::NotGlobalAdd
    }
}
