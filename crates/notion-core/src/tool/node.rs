use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::Command;

use super::{arg0, command_for, NoSuchToolError, Tool};
use crate::session::{ActivityKind, Session};

use notion_fail::Fallible;

/// Represents a Node executable.
pub struct Node(Command);

impl Tool for Node {
    fn new(session: &mut Session) -> Fallible<Self> {
        session.add_event_start(ActivityKind::Node);

        let mut args = args_os();
        let exe = arg0(&mut args)?;
        if let Some(ref platform) = session.current_platform()? {
            let image = platform.checkout(session)?;
            Ok(Self::from_components(&exe, args, &image.path()?))
        } else {
            throw!(NoSuchToolError {
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
