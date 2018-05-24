//! Events for the sessions in executables and shims and everything

extern crate os_info;

use std::env;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use monitor::LazyMonitor;
use notion_fail::{Fallible, NotionError};
use session::ActivityKind;

// the Event data that is serialized to JSON and sent the plugin
#[derive(Serialize)]
pub struct Event {
    timestamp: u64,
    name: String,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<ErrorEnv>,
}

#[derive(Serialize)]
pub struct ErrorEnv {
    argv: String,
    exec_path: String,
    path: String,
    platform: String,
    platform_version: String,
}

enum EventKind {
    Start,
    End,
    Error,
}

impl Display for EventKind {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &EventKind::Start => "start",
            &EventKind::End => "end",
            &EventKind::Error => "error",
        };
        f.write_str(s)
    }
}

impl EventKind {
    pub fn into_event(
        self,
        activity_kind: ActivityKind,
        exit_code: Option<i32>,
        error: Option<&NotionError>,
    ) -> Event {

        Event {
            timestamp: unix_timestamp(),
            name: activity_kind.to_string(),
            event: self.to_string(),
            exit_code: exit_code,
            error: error.map(|e| e.to_string()),
            env: get_error_env(error),
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

fn get_error_env(error: Option<&NotionError>) -> Option<ErrorEnv> {
    if error.is_some() {
        let path = match env::var("PATH") {
            Ok(p) => p,
            Err(_e) => "error: Unable to get path from envirnoment".to_string(),
        };
        let argv = env::args().collect::<Vec<String>>().join(" ");
        let exec_path = match env::current_exe() {
            Ok(ep) => ep.display().to_string(),
            Err(_e) => "error: Unable to get executable path from envirnoment".to_string(),
        };

        let info = os_info::get();
        let platform = info.os_type().to_string();
        let platform_version = info.version().to_string();

        return Some(ErrorEnv {
            argv: argv,
            exec_path: exec_path,
            path: path,
            platform: platform,
            platform_version: platform_version,
        })
    }
    None
}

pub struct EventLog {
    events: Vec<Event>,
    monitor: LazyMonitor,
}

impl EventLog {
    /// Constructs a new 'EventLog'
    pub fn new() -> Fallible<EventLog> {
        Ok(EventLog {
            events: Vec::new(),
            monitor: LazyMonitor::new(),
        })
    }

    pub fn add_event_start(&mut self, activity_kind: ActivityKind) {
        self.add_event(EventKind::Start, activity_kind, None, None)
    }
    pub fn add_event_end(&mut self, activity_kind: ActivityKind, exit_code: Option<i32>) {
        self.add_event(EventKind::End, activity_kind, exit_code, None)
    }
    pub fn add_event_error(&mut self, activity_kind: ActivityKind, error: &NotionError) {
        let exit_code = error.exit_code();
        self.add_event(
            EventKind::Error,
            activity_kind,
            Some(exit_code),
            Some(error),
        )
    }

    fn add_event(
        &mut self,
        event_kind: EventKind,
        activity_kind: ActivityKind,
        exit_code: Option<i32>,
        error: Option<&NotionError>,
    ) {
        let event = event_kind.into_event(activity_kind, exit_code, error);
        self.events.push(event);
    }

    // send the events from this session to the monitor
    pub fn send_events(&mut self, command: Option<String>) {
        self.monitor
            .get_mut(command)
            .unwrap()
            .send_events(&self.events);
    }
}
