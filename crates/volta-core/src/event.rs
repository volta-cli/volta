//! Events for the sessions in executables and shims and everything

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{ExitCode, VoltaError};
use crate::hook::Publish;
use crate::monitor::send_events;
use crate::session::ActivityKind;

// the Event data that is serialized to JSON and sent the plugin
#[derive(Deserialize, Serialize)]
pub struct Event {
    timestamp: u64,
    pub name: String,
    pub event: EventKind,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ErrorEnv {
    argv: String,
    exec_path: String,
    path: String,
    platform: String,
    platform_version: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Start {
        argv: String,
    },
    End {
        exit_code: i32,
    },
    Error {
        exit_code: i32,
        error: String,
        env: ErrorEnv,
    },
    ToolEnd {
        exit_code: i32,
    },
}

impl EventKind {
    pub fn into_event(self, activity_kind: ActivityKind) -> Event {
        Event {
            timestamp: unix_timestamp(),
            name: activity_kind.to_string(),
            event: self,
        }
    }
}

// returns the current number of milliseconds since the epoch
fn unix_timestamp() -> u64 {
    let start = SystemTime::now();
    let duration = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let nanosecs_since_epoch = duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
    nanosecs_since_epoch / 1_000_000
}

fn get_error_env() -> ErrorEnv {
    let path = match env::var("PATH") {
        Ok(p) => p,
        Err(_e) => "error: Unable to get path from environment".to_string(),
    };
    let argv = env::args().collect::<Vec<String>>().join(" ");
    let exec_path = match env::current_exe() {
        Ok(ep) => ep.display().to_string(),
        Err(_e) => "error: Unable to get executable path from environment".to_string(),
    };

    let info = os_info::get();
    let platform = info.os_type().to_string();
    let platform_version = info.version().to_string();

    ErrorEnv {
        argv,
        exec_path,
        path,
        platform,
        platform_version,
    }
}

pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    /// Constructs a new 'EventLog'
    pub fn init() -> Self {
        EventLog { events: Vec::new() }
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        let argv = env::args_os()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        self.add_event(EventKind::Start { argv }, activity_kind)
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: ExitCode) {
        self.add_event(
            EventKind::End {
                exit_code: exit_code as i32,
            },
            activity_kind,
        )
    }
    pub fn add_event_tool_end(&mut self, activity_kind: ActivityKind, exit_code: i32) {
        self.add_event(EventKind::ToolEnd { exit_code }, activity_kind)
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &VoltaError) {
        self.add_event(
            EventKind::Error {
                exit_code: error.exit_code() as i32,
                error: error.to_string(),
                env: get_error_env(),
            },
            activity_kind,
        )
    }

    fn add_event(&mut self, event_kind: EventKind, activity_kind: ActivityKind) {
        let event = event_kind.into_event(activity_kind);
        self.events.push(event);
    }

    pub fn publish(&self, plugin: Option<&Publish>) {
        match plugin {
            // Note: This call to unimplemented is left in, as it's not a Fallible operation that can use ErrorKind::Unimplemented
            Some(&Publish::Url(_)) => unimplemented!(),
            Some(&Publish::Bin(ref command)) => {
                send_events(command, &self.events);
            }
            None => {}
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::{EventKind, EventLog};
    use crate::error::{ErrorKind, ExitCode};
    use crate::session::ActivityKind;
    use regex::Regex;

    #[test]
    fn test_adding_events() {
        let mut event_log = EventLog::init();
        assert_eq!(event_log.events.len(), 0);

        event_log.add_event_start(ActivityKind::Current);
        assert_eq!(event_log.events.len(), 1);
        assert_eq!(event_log.events[0].name, "current");
        match event_log.events[0].event {
            EventKind::Start { ref argv } => {
                let re = Regex::new("volta_core").unwrap();
                assert!(re.is_match(argv));
            }
            _ => {
                panic!(
                    "Expected EventKind::Start {{ argv }}, Got: {:?}",
                    event_log.events[0].event
                );
            }
        }

        event_log.add_event_end(ActivityKind::Pin, ExitCode::NetworkError);
        assert_eq!(event_log.events.len(), 2);
        assert_eq!(event_log.events[1].name, "pin");
        assert_eq!(event_log.events[1].event, EventKind::End { exit_code: 5 });

        event_log.add_event_tool_end(ActivityKind::Version, 12);
        assert_eq!(event_log.events.len(), 3);
        assert_eq!(event_log.events[2].name, "version");
        assert_eq!(
            event_log.events[2].event,
            EventKind::ToolEnd { exit_code: 12 }
        );

        let error = ErrorKind::BinaryExecError.into();
        event_log.add_event_error(ActivityKind::Install, &error);
        assert_eq!(event_log.events.len(), 4);
        assert_eq!(event_log.events[3].name, "install");
        // not checking the error because it has too much machine-specific info
    }
}
