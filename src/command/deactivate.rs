use notion_core::env;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{Fallible, ResultExt};

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
Remove Notion from the current shell

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

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Deactivate);
        match self {
            Deactivate::Help => {
                Help::Command(CommandName::Deactivate).run(session)?;
            }
            Deactivate::Deactivate => {
                // FIXME: proper escaping
                let path_var = env::path_for_system_node().into_string().unknown()?;
                env::write_postscript(format!("export PATH='{}'\n", path_var))?;
            }
        };
        session.add_event_end(ActivityKind::Deactivate, 0);
        Ok(true)
    }
}
