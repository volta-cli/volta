use std::env;

use structopt::StructOpt;
use which::which_in;

use notion_core::platform::System;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{ExitCode, Fallible, ResultExt};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Which {
    /// The binary to find, e.g. `node` or `npm`
    binary: String,
}

impl Command for Which {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Which);

        // Treat any error with obtaining the current platform image as if the image doesn't exist
        // However, errors in obtaining the current working directory or the System path should
        // still be treated as errors.
        let cwd = env::current_dir().unknown()?;
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
            }
            Err(_) => {
                // `which_in` Will return an Err if it can't find the binary in the path
                // In that case, we want to do nothing, instead of showing the user an error
            }
        }

        session.add_event_end(ActivityKind::Which, ExitCode::Success);
        Ok(())
    }
}
