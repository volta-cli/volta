use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::vec::Vec;

use lazycell::LazyCell;
use serde_json;

use event::Event;

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
    // if plugin command is not configured, this is a no-op
    pub fn send_events(&mut self, events: &Vec<Event>) -> () {
        if let Some(ref mut child_process) = self.monitor_process {
            let p_stdin = child_process.stdin.as_mut().unwrap();

            let json = serde_json::to_string(&events);
            match json {
                Ok(data) => {
                    // FIXME: tighten up this error message
                    write!(p_stdin, "{}", data).expect("Writing data to plugin failed!");
                }
                Err(error) => {
                    // FIXME: tighten up this error message
                    eprintln!("There was a problem serializing the JSON data: {:?}", error);
                }
            };
        }
    }
}

pub struct LazyMonitor {
    monitor: LazyCell<Monitor>,
}

impl LazyMonitor {
    /// Constructs a new `LazyMonitor`.
    pub fn new() -> LazyMonitor {
        LazyMonitor {
            monitor: LazyCell::new(),
        }
    }

    /// Forces creating a monitor and returns an immutable reference to it.
    pub fn get(&self, command: &str) -> &Monitor {
        self.monitor.borrow_with(|| Monitor::new(command))
    }

    /// Forces creating a monitor and returns a mutable reference to it.
    pub fn get_mut(&mut self, command: &str) -> &mut Monitor {
        self.monitor
            .borrow_mut_with(|| Monitor::new(command))
    }
}

fn spawn_process(command: &str) -> Option<Child> {
    command.split(" ").take(1).next().and_then(|executable| {
        let child = Command::new(executable)
                    .args(command.split(" ").skip(1))
                    .stdin(Stdio::piped()) // JSON data is sent over stdin
                    // .stdout(Stdio::piped()) // let the plugin write to stdout for now
                    .spawn();
        match child {
            Err(err) => {
                eprintln!("Error running plugin command: '{}'", command);
                eprintln!("{}", err);
                None
            }
            Ok(c) => Some(c),
        }
    })
}
