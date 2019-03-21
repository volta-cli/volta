use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::Command;

use super::{command_for, intercept_global_installs, Tool};
use crate::error::ErrorDetails;
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

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Self::from_components(
                OsStr::new("yarn"),
                args,
                &image.path()?,
            ))
        } else {
            throw!(ErrorDetails::NoSuchTool {
                tool: "Yarn".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Yarn(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
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
