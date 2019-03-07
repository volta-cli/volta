use std::env;

use structopt::StructOpt;
use which::which_in;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{throw, ExitCode, Fallible, ResultExt};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Which {
    /// The binary to find, e.g. `node` or `npm`
    binary: String,
}

impl Command for Which {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Which);

        let platform = session.current_platform()?;

        match platform {
            Some(platform) => {
                let image = platform.checkout(session)?;
                let path = image.path()?;
                let cwd = env::current_dir().unknown()?;

                match which_in(&self.binary, Some(path), cwd) {
                    Ok(result) => {
                        println!("{}", result.to_string_lossy());
                    }
                    Err(_) => {
                        // `which_in` Will return an Err if it can't find the binary in the path.
                        // In that case, we want to do nothing, instead of showing the user an error
                    }
                };
            }
            None => throw!(ErrorDetails::NoPlatformSpecified),
        }

        session.add_event_end(ActivityKind::Which, ExitCode::Success);
        Ok(())
    }
}
