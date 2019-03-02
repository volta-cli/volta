use structopt::StructOpt;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub enum Config {
    #[structopt(name = "get")]
    Get { key: String },

    #[structopt(name = "set")]
    Set { key: String, value: String },

    #[structopt(name = "delete")]
    Delete { key: String },

    #[structopt(name = "list")]
    List,

    #[structopt(name = "edit")]
    Edit,
}

impl Command for Config {
    fn run(self, session: &mut Session) -> Fallible<()> {
        session.add_event_start(ActivityKind::Version);

        let result = match self {
            Config::Get { key: _ } => Ok(()),
            Config::Set { key: _, value: _ } => throw!(ErrorDetails::CommandNotImplemented {
                command_name: "set".to_string()
            }),
            Config::Delete { key: _ } => throw!(ErrorDetails::CommandNotImplemented {
                command_name: "delete".to_string()
            }),
            Config::List => throw!(ErrorDetails::CommandNotImplemented {
                command_name: "list".to_string()
            }),
            Config::Edit => throw!(ErrorDetails::CommandNotImplemented {
                command_name: "edit".to_string()
            }),
        };

        session.add_event_end(ActivityKind::Version, ExitCode::Success);

        result
    }
}
