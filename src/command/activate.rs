use notion_core::image::System;
use notion_core::session::{ActivityKind, Session};
use notion_core::shell::{CurrentShell, Postscript, Shell};
use notion_fail::{ExitCode, Fallible};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args;

pub(crate) enum Activate {
    Help,
    Activate
}

impl Command for Activate {
    type Args = Args;

    const USAGE: &'static str = "
Re-Enable Notion in the current shell

Usage:
    notion activate
    notion activate -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Activate::Help
    }

    fn parse(_: Notion, _:Args) -> Fallible<Self> {
        Ok(Activate::Activate)
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Activate);
        match self {
            Activate::Help => {
                Help::Command(CommandName::Activate).run(session)?;
            }
            Activate::Activate => {
                let shell = CurrentShell::detect()?;

                let postscript = match System::enabled_path()?.into_string() {
                    Ok(path) => Postscript::Activate(path),
                    Err(_) => unimplemented!(),
                };

                shell.save_postscript(&postscript)?;
            }
        }
        session.add_event_end(ActivityKind::Activate, ExitCode::Success);
        Ok(())
    }
}