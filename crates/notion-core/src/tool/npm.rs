use std::env::args_os;
use std::ffi::{OsStr, OsString};

use super::{intercept_global_installs, ToolCommand};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use notion_fail::{throw, Fallible};

pub(super) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Npm);

    match session.current_platform()? {
        Some(ref platform) => {
            if intercept_global_installs() && is_global_npm_install() {
                throw!(ErrorDetails::NoGlobalInstalls);
            }
            let image = platform.checkout(session)?;
            let path = image.path()?;
            Ok(ToolCommand::direct(OsStr::new("npm"), args, &path))
        }
        None => ToolCommand::passthrough(OsStr::new("npm"), args, ErrorDetails::NoPlatform),
    }
}

fn is_global_npm_install() -> bool {
    let command = args_os()
        .skip(1)
        .skip_while(|arg| match arg.to_str() {
            Some(arg) => arg.starts_with("-"),
            None => false,
        })
        .next();

    // npm global installs will have the command `i`, `install`, `add` or `isntall`
    // See https://github.com/npm/cli/blob/latest/lib/config/cmd-list.js
    // Additionally, they will have `-g` or `--global` somewhere in the argument list
    if command == Some(OsString::from("install"))
        || command == Some(OsString::from("i"))
        || command == Some(OsString::from("isntall"))
        || command == Some(OsString::from("add"))
    {
        args_os().any(|arg| arg == OsString::from("-g") || arg == OsString::from("--global"))
    } else {
        false
    }
}
