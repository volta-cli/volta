use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::vec::Vec;

use lazycell::LazyCell;
use serde_json;

use event::Event;
use notion_fail::Fallible;

pub struct Monitor {
    monitor_process: Option<Child>,
}

impl Monitor {
    /// Returns the current monitor.
    pub fn new(command: Option<String>) -> Monitor {
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
                    write!(p_stdin, "{}", data).expect("Writing data to plugin failed!");
                }
                Err(error) => {
                    println!("There was a problem serializing the JSON data: {:?}", error);
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
    pub fn get(&self, command: Option<String>) -> Fallible<&Monitor> {
        self.monitor.try_borrow_with(|| Ok(Monitor::new(command)))
    }

    /// Forces creating a monitor and returns a mutable reference to it.
    pub fn get_mut(&mut self, command: Option<String>) -> Fallible<&mut Monitor> {
        self.monitor
            .try_borrow_mut_with(|| Ok(Monitor::new(command)))
    }
}

fn spawn_process(command: Option<String>) -> Option<Child> {
    if let Some(ref cmd) = command {
        // TODO
        // - split the command and arguments
        // - run the command with CWD of notion?
        // https://stackoverflow.com/questions/26643688/how-do-i-split-a-string-in-rust

        let child = Command::new(cmd)
            // .arg()
            .stdin(Stdio::piped()) // JSON data is sent over stdin
            // .stdout(Stdio::piped()) // let the plugin write to stdout and stderr, or not?
            .spawn();
        return match child {
            Err(err) => {
                eprintln!("Error running plugin command: '{}'", cmd);
                eprintln!("{}", err);
                None
            }
            Ok(c) => Some(c)
        }
    }
    None
}
