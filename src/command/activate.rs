use structopt::StructOpt;

use notion_core::platform::System;
use notion_core::session::{ActivityKind, Session};
use notion_core::shell::{CurrentShell, Postscript, Shell};
use notion_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Activate {}

impl Command for Activate {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Activate);
        let shell = CurrentShell::detect()?;

        let postscript = match System::enabled_path()?.into_string() {
            Ok(path) => Postscript::Activate(path),
            Err(_) => unimplemented!(),
        };

        shell.save_postscript(&postscript)?;
        session.add_event_end(ActivityKind::Activate, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
