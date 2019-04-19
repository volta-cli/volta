use std::env::ArgsOs;
use std::ffi::OsStr;
use std::process::Command;

use super::{command_for, Tool};
use crate::error::ErrorDetails;
use crate::session::{ActivityKind, Session};

use jetson_fail::{throw, Fallible};

/// Represents a Node executable.
pub struct Node(Command);

impl Tool for Node {
    type Arguments = ArgsOs;

    fn new(args: ArgsOs, session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Node);

        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Node(command_for(OsStr::new("node"), args, &image.path()?)))
        } else {
            throw!(ErrorDetails::NoPlatform);
        }
    }

    fn command(self) -> Command {
        self.0
    }
}
