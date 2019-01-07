use notion_core::platform::System;
use notion_core::session::{ActivityKind, Session};
use notion_core::shell::{CurrentShell, Postscript, Shell};
use notion_fail::{ExitCode, Fallible};

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args;

pub(crate) enum Deactivate {
    Help,
    Deactivate,
}

impl Command for Deactivate {
    type Args = Args;

    const USAGE: &'static str = "
Disable Notion in the current shell

Usage:
    notion deactivate
    notion deactivate -h | --help

Options:
    -h, --help     Display this message
";

    fn help() -> Self {
        Deactivate::Help
    }

    fn parse(_: Notion, _: Args) -> Fallible<Self> {
        Ok(Deactivate::Deactivate)
    }

    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Deactivate);
        match self {
            Deactivate::Help => {
                Help::Command(CommandName::Deactivate).run(session)?;
            }
            Deactivate::Deactivate => {
                let shell = CurrentShell::detect()?;

                let postscript = match System::path()?.into_string() {
                    Ok(path) => Postscript::Deactivate(path),
                    Err(_) => unimplemented!(),
                };

                shell.save_postscript(&postscript)?;
            }
        };
        session.add_event_end(ActivityKind::Deactivate, ExitCode::Success);
        Ok(())
    }
}
