use std::env::ArgsOs;
use std::ffi::OsStr;
use std::process::Command;

use super::{command_for, Tool};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use notion_fail::{throw, Fallible};

/// Represents a Node executable.
pub struct Node(Command);

impl Tool for Node {
    type Arguments = ArgsOs;

    fn new(args: ArgsOs, session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Node);

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Self::from_components(
                OsStr::new("node"),
                args,
                &image.path()?,
            ))
        } else {
            throw!(ErrorDetails::NoSuchTool {
                tool: "Node".to_string()
            });
        }
    }

    fn from_components(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Node(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        self.0
    }
}
