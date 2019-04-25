use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use notion_fail::ExitCode;

const NODE_VERSION_INFO: &'static str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v9.27.6","npm":"5.6.17","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v8.9.10","npm":"5.6.7","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v6.19.62","npm":"3.10.1066","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
]
"#;

#[test]
fn no_cause_shown_if_no_verbose_flag() {
    let s = sandbox()
        .env_remove("NOTION_DEV")
        .node_available_versions(NODE_VERSION_INFO)
        .build();

    assert_that!(
        s.notion("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_does_not_contain("cause[..]")
    );
}

#[test]
fn cause_shown_if_verbose_flag() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    assert_that!(
        s.notion("install node@10 --verbose"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("cause[..]")
    );
}

#[test]
fn no_cause_if_no_underlying_error() {
    let s = sandbox().build();

    assert_that!(
        s.notion("use --verbose"),
        execs()
            .with_status(ExitCode::InvalidArguments as i32)
            .with_stderr_does_not_contain("cause[..]")
    );
}

#[test]
fn error_log_if_underlying_cause() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    assert_that!(
        s.notion("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("Error details written to[..]")
    );

    println!("ROOT DIRECTORY: {:?}", s.root());
    let mut log_dir_contents = s.read_log_dir().expect("Could not read log directory");
    assert_that!(log_dir_contents.next(), some());
}

#[test]
fn no_error_log_if_no_underlying_cause() {
    let s = sandbox().build();

    assert_that!(
        s.notion("use"),
        execs()
            .with_status(ExitCode::InvalidArguments as i32)
            .with_stderr_does_not_contain("Error details written to[..]")
    );

    // The log directory may not exist at all. If so, we know we didn't write to it
    if let Some(mut log_dir_contents) = s.read_log_dir() {
        assert_that!(log_dir_contents.next(), none());
    }
}
