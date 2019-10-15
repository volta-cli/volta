use log::warn;
use structopt::StructOpt;

use volta_core::error::ErrorDetails;
use volta_core::platform::System;
use volta_core::session::{ActivityKind, Session};
use volta_core::shell::{CurrentShell, Postscript, Shell};
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Deactivate {}

impl Command for Deactivate {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Deactivate);
        let shell = CurrentShell::detect()?;

        let path = System::path()?
            .into_string()
            .map_err(|_| ErrorDetails::Unimplemented {
                feature: "volta deactivate".into(),
            })?;
        let postscript = Postscript::Deactivate(path);

        shell.save_postscript(&postscript)?;

        warn!("`volta deactivate` is deprecated and will be removed in a future version.");

        session.add_event_end(ActivityKind::Deactivate, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
