use structopt::StructOpt;

use notion_core::platform::System;
use notion_core::session::{ActivityKind, Session};
use notion_core::shell::{CurrentShell, Postscript, Shell};
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Deactivate {}

impl Command for Deactivate {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Deactivate);
        let shell = CurrentShell::detect()?;

        let postscript = match System::path()?.into_string() {
            Ok(path) => Postscript::Deactivate(path),
            Err(_) => unimplemented!(),
        };

        shell.save_postscript(&postscript)?;
        session.add_event_end(ActivityKind::Deactivate, ExitCode::Success);
        Ok(())
    }
}
