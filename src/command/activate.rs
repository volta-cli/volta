use log::warn;
use structopt::StructOpt;

use volta_core::error::ErrorDetails;
use volta_core::platform::System;
use volta_core::session::{ActivityKind, Session};
use volta_core::shell::{CurrentShell, Postscript, Shell};
use volta_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Activate {}

impl Command for Activate {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Activate);
        let shell = CurrentShell::detect()?;

        let path =
            System::enabled_path()?
                .into_string()
                .map_err(|_| ErrorDetails::Unimplemented {
                    feature: "volta activate".into(),
                })?;
        let postscript = Postscript::Activate(path);

        shell.save_postscript(&postscript)?;

        warn!("`volta activate` is deprecated and will be removed in a future version.");

        session.add_event_end(ActivityKind::Activate, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
