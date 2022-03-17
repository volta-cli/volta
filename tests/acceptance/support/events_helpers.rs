use std::fs::File;

use crate::support::sandbox::Sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;

use volta_core::event::{Event, EventKind};

pub enum EventKindMatcher<'a> {
    Start,
    End { exit_code: i32 },
    Error { exit_code: i32, error: &'a str },
    ToolEnd { exit_code: i32 },
    Args { argv: &'a str },
}

pub fn match_start() -> EventKindMatcher<'static> {
    EventKindMatcher::Start
}

pub fn match_error(exit_code: i32, error: &str) -> EventKindMatcher {
    EventKindMatcher::Error { exit_code, error }
}

pub fn match_end(exit_code: i32) -> EventKindMatcher<'static> {
    EventKindMatcher::End { exit_code }
}

pub fn match_tool_end(exit_code: i32) -> EventKindMatcher<'static> {
    EventKindMatcher::ToolEnd { exit_code }
}

pub fn match_args(argv: &str) -> EventKindMatcher {
    EventKindMatcher::Args { argv }
}

pub fn assert_events(sandbox: &Sandbox, matchers: Vec<(&str, EventKindMatcher)>) {
    let events_path = sandbox.root().join("events.json");
    assert_that!(&events_path, file_exists());

    let events_file = File::open(events_path).expect("Error reading 'events.json' file in sandbox");
    let events: Vec<Event> = serde_json::de::from_reader(events_file)
        .expect("Error parsing 'events.json' file in sandbox");
    assert_that!(events.len(), eq(matchers.len()));

    for (i, matcher) in matchers.iter().enumerate() {
        assert_that!(&events[i].name, eq(matcher.0));
        match matcher.1 {
            EventKindMatcher::Start => {
                assert_that!(&events[i].event, eq(&EventKind::Start));
            }
            EventKindMatcher::End {
                exit_code: expected_exit_code,
            } => {
                if let EventKind::End { exit_code } = &events[i].event {
                    assert_that!(*exit_code, eq(expected_exit_code));
                } else {
                    panic!(
                        "Expected: End {{ exit_code: {} }}, Got: {:?}",
                        expected_exit_code, events[i].event
                    );
                }
            }
            EventKindMatcher::Error {
                exit_code: expected_exit_code,
                error: expected_error,
            } => {
                if let EventKind::Error {
                    exit_code, error, ..
                } = &events[i].event
                {
                    assert_that!(*exit_code, eq(expected_exit_code));
                    assert_that!(error.clone(), matches_regex(expected_error));
                } else {
                    panic!(
                        "Expected: Error {{ exit_code: {}, error: {} }}, Got: {:?}",
                        expected_exit_code, expected_error, events[i].event
                    );
                }
            }
            EventKindMatcher::ToolEnd {
                exit_code: expected_exit_code,
            } => {
                if let EventKind::End { exit_code } = &events[i].event {
                    assert_that!(*exit_code, eq(expected_exit_code));
                } else {
                    panic!(
                        "Expected: ToolEnd {{ exit_code: {} }}, Got: {:?}",
                        expected_exit_code, events[i].event
                    );
                }
            }
            EventKindMatcher::Args {
                argv: expected_argv,
            } => {
                if let EventKind::Args { argv } = &events[i].event {
                    assert_that!(argv.clone(), matches_regex(expected_argv));
                } else {
                    panic!(
                        "Expected: Args {{ argv: {} }}, Got: {:?}",
                        expected_argv, events[i].event
                    );
                }
            }
        }
    }
}
