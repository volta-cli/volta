use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};

use super::{
    arg0, command_for, display_error, intercept_global_installs, NoGlobalInstallError,
    NoSuchToolError, Tool,
};
use crate::session::{ActivityKind, Session};

use notion_fail::{throw, Fallible};

/// Represents a Yarn executable.
pub struct Yarn(Command);

impl Tool for Yarn {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Yarn);

        let mut args = args_os();
        let exe = arg0(&mut args)?;

        if intercept_global_installs() && is_global_yarn_add() {
            throw!(NoGlobalInstallError);
        }

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Self::from_components(&exe, args, &image.path()?))
        } else {
            throw!(NoSuchToolError {
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

    /// Perform any tasks which must be run after the tool runs but before exiting.
    fn finalize(session: &Session, maybe_status: &io::Result<ExitStatus>) {
        if let Ok(_) = maybe_status {
            if let Ok(Some(project)) = session.project() {
                let errors = project.autoshim();

                for error in errors {
                    display_error(&error);
                }
            }
        }
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
