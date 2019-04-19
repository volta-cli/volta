use structopt::StructOpt;

use jetson_core::platform::System;
use jetson_core::session::{ActivityKind, Session};
use jetson_core::shell::{CurrentShell, Postscript, Shell};
use jetson_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Deactivate {}

impl Command for Deactivate {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Deactivate);
        let shell = CurrentShell::detect()?;

        let postscript = match System::path()?.into_string() {
            Ok(path) => Postscript::Deactivate(path),
            Err(_) => unimplemented!(),
        };

        shell.save_postscript(&postscript)?;
        session.add_event_end(ActivityKind::Deactivate, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
