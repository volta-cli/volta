use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::Command;
use std::rc::Rc;

use super::{command_for, intercept_global_installs, Tool};
use crate::error::ErrorDetails;
use crate::platform::PlatformSpec;
use crate::session::{ActivityKind, Session};

use notion_fail::{throw, Fallible};

/// Represents a Yarn executable.
pub struct Yarn(Command);

impl Tool for Yarn {
    type Arguments = ArgsOs;

    fn new(args: ArgsOs, session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Yarn);

        if intercept_global_installs() && is_global_yarn_add() {
            throw!(ErrorDetails::NoGlobalInstalls);
        }

        let platform = get_yarn_platform(session)?;
        let image = platform.checkout(session)?;

        Ok(Self::from_components(
            OsStr::new("yarn"),
            args,
            &image.path()?,
        ))
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Yarn(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}

/// Determine the correct platform (project or user) and check if yarn is set for that platform
fn get_yarn_platform(session: &mut Session) -> Fallible<Rc<PlatformSpec>> {
    // First check if we are in a pinned project
    if let Some(platform) = session.project_platform()? {
        return match platform.yarn {
            Some(_) => Ok(platform),
            None => Err(ErrorDetails::NoProjectYarn.into()),
        };
    }

    // If not, fall back to the user platform
    if let Some(platform) = session.user_platform()? {
        return match platform.yarn {
            Some(_) => Ok(platform),
            None => Err(ErrorDetails::NoUserYarn.into()),
        };
    }

    throw!(ErrorDetails::NoPlatform);
}

fn is_global_yarn_add() -> bool {
    // Yarn global installs must be of the form `yarn global add`
    // However, they may have options intermixed, e.g. yarn --verbose global add ember-cli
    args_os()
        .skip(1)
        .filter(|arg| match arg.to_str() {
            Some(arg) => !arg.starts_with("-"),
            None => true,
        })
        .take(2)
        .eq(vec!["global", "add"])
}
