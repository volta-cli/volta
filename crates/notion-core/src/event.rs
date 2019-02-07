//! Events for the sessions in executables and shims and everything

use os_info;

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::monitor::LazyMonitor;
use crate::plugin::Publish;
use crate::session::ActivityKind;
use notion_fail::{ExitCode, NotionError};

// the Event data that is serialized to JSON and sent the plugin
#[derive(Serialize)]
pub struct Event {
    timestamp: u64,
    name: String,
    event: EventKind,
}

#[derive(Serialize)]
pub struct ErrorEnv {
    argv: String,
    exec_path: String,
    path: String,
    platform: String,
    platform_version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum EventKind {
    Start,
    End {
        exit_code: ExitCode,
    },
    Error {
        exit_code: ExitCode,
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

    return ErrorEnv {
        argv: argv,
        exec_path: exec_path,
        path: path,
        platform: platform,
        platform_version: platform_version,
    };
}

pub struct EventLog {
    events: Vec<Event>,
    monitor: LazyMonitor,
}

impl EventLog {
    /// Constructs a new 'EventLog'
    pub fn new() -> Self {
        EventLog {
            events: Vec::new(),
            monitor: LazyMonitor::new(),
        }
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        self.add_event(EventKind::Start, activity_kind)
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: ExitCode) {
        self.add_event(EventKind::End { exit_code }, activity_kind)
    }
    pub fn add_event_tool_end(&mut self, activity_kind: ActivityKind, exit_code: i32) {
        self.add_event(EventKind::ToolEnd { exit_code }, activity_kind)
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &NotionError) {
        let exit_code = error.exit_code();
        self.add_event(
            EventKind::Error {
                exit_code: exit_code,
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

    pub fn publish(&mut self, plugin: Option<&Publish>) {
        match plugin {
            Some(&Publish::Url(_)) => unimplemented!(),
            Some(&Publish::Bin(ref command)) => {
                self.monitor.get_mut(command).send_events(&self.events);
            }
            None => {}
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::EventLog;
    use crate::session::ActivityKind;
    use notion_fail::{ExitCode, FailExt};
    use std::io;

    #[test]
    fn test_adding_events() {
        let mut event_log = EventLog::new();
        assert_eq!(event_log.events.len(), 0);

        event_log.add_event_start(ActivityKind::Current);
        assert_eq!(event_log.events.len(), 1);
        assert_eq!(event_log.events[0].name, "current");

        event_log.add_event_end(ActivityKind::Pin, ExitCode::NetworkError);
        assert_eq!(event_log.events.len(), 2);
        assert_eq!(event_log.events[1].name, "pin");

        event_log.add_event_tool_end(ActivityKind::Version, 12);
        assert_eq!(event_log.events.len(), 3);
        assert_eq!(event_log.events[2].name, "version");

        let error = io::Error::new(io::ErrorKind::Other, "test!").unknown();
        event_log.add_event_error(ActivityKind::Install, &error);
        assert_eq!(event_log.events.len(), 4);
        assert_eq!(event_log.events[3].name, "install");
    }
}
