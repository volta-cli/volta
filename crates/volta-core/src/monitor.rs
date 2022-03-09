use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Stdio};

use log::debug;
use tempfile::NamedTempFile;

use crate::command::create_command;
use crate::event::Event;

/// send event to the spawned command process
// if hook command is not configured, this is not called
pub fn send_events(command: &str, events: &[Event]) {
    // TODO: make these expects into errors or whatever
    let events_json =
        serde_json::to_string_pretty(&events).expect("Problem serializing events JSON data");
    let mut events_file = NamedTempFile::new().expect("Could not create temp file for events");
    events_file
        .write_all(events_json.as_bytes())
        .expect("Writing data to file failed");
    // don't automatically delete this temp file please
    let path = events_file.into_temp_path();
    let tempfile_path = path.keep().expect("Could not persist temp file");

    // spawn a child process, with the path to that temp file as an env var
    if let Some(ref mut child_process) = spawn_process(command, tempfile_path) {
        if let Some(ref mut p_stdin) = child_process.stdin.as_mut() {
            // still send the data over stdin
            writeln!(p_stdin, "{}", events_json).expect("Writing data to plugin failed!");
        }
    }
}

fn spawn_process(command: &str, tempfile_path: PathBuf) -> Option<Child> {
    command.split(' ').take(1).next().and_then(|executable| {
        let mut child = create_command(executable);
        child.args(command.split(' ').skip(1));
        child.stdin(Stdio::piped());
        child.env("EVENTS_FILE", tempfile_path);

        #[cfg(not(debug_assertions))]
        // Hide stdout and stderr of spawned process in release mode
        child.stdout(Stdio::null()).stderr(Stdio::null());

        match child.spawn() {
            Err(err) => {
                debug!("Unable to run plugin command: '{}'\n{}", command, err);
                None
            }
            Ok(c) => Some(c),
        }
    })
}
