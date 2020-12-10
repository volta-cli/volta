use std::io::Write;
use std::process::{Child, Stdio};

use log::error;

use crate::command::create_command;
use crate::event::Event;

pub struct Monitor {
    monitor_process: Option<Child>,
}

impl Monitor {
    /// Returns the current monitor.
    pub fn new(command: &str) -> Monitor {
        Monitor {
            monitor_process: spawn_process(command),
        }
    }

    /// send event to the monitor process
    // if hook command is not configured, this is a no-op
    pub fn send_events(&mut self, events: &[Event]) {
        if let Some(ref mut child_process) = self.monitor_process {
            if let Some(ref mut p_stdin) = child_process.stdin.as_mut() {
                let json = serde_json::to_string(&events);

                match json {
                    Ok(data) => {
                        // FIXME: tighten up this error message
                        write!(p_stdin, "{}", data).expect("Writing data to plugin failed!");
                    }
                    Err(error) => {
                        // FIXME: tighten up this error message
                        error!("There was a problem serializing the JSON data: {:?}", error);
                    }
                };
            }
        }
    }
}

fn spawn_process(command: &str) -> Option<Child> {
    command.split(' ').take(1).next().and_then(|executable| {
        let mut child = create_command(executable);
        child.args(command.split(' ').skip(1));
        child.stdin(Stdio::piped());

        #[cfg(not(debug_assertions))]
        // Hide stdout and stderr of spawned process in release mode
        child.stdout(Stdio::null()).stderr(Stdio::null());

        match child.spawn() {
            Err(err) => {
                error!("Unable to run plugin command: '{}'\n{}", command, err);
                None
            }
            Ok(c) => Some(c),
        }
    })
}
