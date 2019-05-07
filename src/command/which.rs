use std::env;

use structopt::StructOpt;
use which::which_in;

use volta_core::error::ErrorDetails;
use volta_core::platform::System;
use volta_core::session::{ActivityKind, Session};
use volta_fail::{ExitCode, Fallible, ResultExt};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Which {
    /// The binary to find, e.g. `node` or `npm`
    binary: String,
}

impl Command for Which {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Which);

        // Treat any error with obtaining the current platform image as if the image doesn't exist
        // However, errors in obtaining the current working directory or the System path should
        // still be treated as errors.
        let cwd = env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?;
        let path = match session
            .current_platform()
            .unwrap_or(None)
            .and_then(|platform| platform.checkout(session).ok())
            .and_then(|image| image.path().ok())
        {
            Some(path) => path,
            None => System::path()?,
        };

        match which_in(&self.binary, Some(path), cwd) {
            Ok(result) => {
                println!("{}", result.to_string_lossy());
                session.add_event_end(ActivityKind::Which, ExitCode::Success);

                Ok(ExitCode::Success)
            }
            Err(_) => {
                // `which_in` Will return an Err if it can't find the binary in the path
                // In that case, we don't want to print anything out, but we want to return
                // Exit Code 1 (ExitCode::UnknownError)
                session.add_event_end(ActivityKind::Which, ExitCode::UnknownError);
                Ok(ExitCode::UnknownError)
            }
        }
    }
}
