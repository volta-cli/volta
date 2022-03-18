use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Stdio};

use log::debug;
use tempfile::NamedTempFile;

use crate::command::create_command;
use crate::event::Event;

/// Send event to the spawned command process
// if hook command is not configured, this is not called
pub fn send_events(command: &str, events: &[Event]) {
    match serde_json::to_string_pretty(&events) {
        Ok(events_json) => {
            let tempfile_path = env::var_os("VOLTA_WRITE_EVENTS_FILE")
                .and_then(|_| write_events_file(events_json.clone()));
            if let Some(ref mut child_process) = spawn_process(command, tempfile_path) {
                if let Some(ref mut p_stdin) = child_process.stdin.as_mut() {
                    if let Err(error) = writeln!(p_stdin, "{}", events_json) {
                        debug!("Could not write events to executable stdin: {:?}", error);
                    }
                }
            }
        }
        Err(error) => {
            debug!("Could not serialize events data to JSON: {:?}", error);
        }
    }
}

// Write the events JSON to a file in the temporary directory
fn write_events_file(events_json: String) -> Option<PathBuf> {
    match NamedTempFile::new() {
        Ok(mut events_file) => {
            match events_file.write_all(events_json.as_bytes()) {
                Ok(()) => {
                    let path = events_file.into_temp_path();
                    // if it's not persisted, the temp file will be automatically deleted
                    // (and the executable won't be able to read it)
                    match path.keep() {
                        Ok(tempfile_path) => Some(tempfile_path),
                        Err(error) => {
                            debug!("Failed to persist temp file for events data: {:?}", error);
                            None
                        }
                    }
                }
                Err(error) => {
                    debug!("Failed to write events to the temp file: {:?}", error);
                    None
                }
            }
        }
        Err(error) => {
            debug!("Failed to create a temp file for events data: {:?}", error);
            None
        }
    }
}

// Spawn a child process to receive the events data, setting the path to the events file as an env var
fn spawn_process(command: &str, tempfile_path: Option<PathBuf>) -> Option<Child> {
    command.split(' ').take(1).next().and_then(|executable| {
        let mut child = create_command(executable);
        child.args(command.split(' ').skip(1));
        child.stdin(Stdio::piped());
        if let Some(events_file) = tempfile_path {
            child.env("EVENTS_FILE", events_file);
        }

        #[cfg(not(debug_assertions))]
        // Hide stdout and stderr of spawned process in release mode
        child.stdout(Stdio::null()).stderr(Stdio::null());

        match child.spawn() {
            Err(err) => {
                debug!("Unable to run executable command: '{}'\n{}", command, err);
                None
            }
            Ok(c) => Some(c),
        }
    })
}
