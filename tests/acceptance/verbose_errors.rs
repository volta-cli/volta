use crate::support::sandbox::sandbox;
use ci_info::types::{CiInfo, Vendor};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

const NODE_VERSION_INFO: &str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip","linux-arm64"]},
{"version":"v9.27.6","npm":"5.6.17","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip","linux-arm64"]},
{"version":"v8.9.10","npm":"5.6.7","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip","linux-arm64"]},
{"version":"v6.19.62","npm":"3.10.1066","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip","linux-arm64"]}
]
"#;

#[test]
fn no_cause_shown_if_no_verbose_flag() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    // Mock `is_ci` to false so that this works even when running in Volta's CI Test Suite
    ci_info::mock_ci(&CiInfo::new());

    assert_that!(
        s.volta("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_does_not_contain("[..]Error cause[..]")
    );
}

#[test]
fn cause_shown_if_verbose_flag() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    // Mock `is_ci` to false so that this correctly tests the verbose flag
    ci_info::mock_ci(&CiInfo::new());

    assert_that!(
        s.volta("install node@10 --verbose"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Error cause[..]")
    );
}

#[test]
fn no_cause_if_no_underlying_error() {
    let s = sandbox().build();

    assert_that!(
        s.volta("use --verbose"),
        execs()
            .with_status(ExitCode::InvalidArguments as i32)
            .with_stderr_does_not_contain("[..]Error cause[..]")
    );
}

#[test]
fn error_log_if_underlying_cause() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    // Mock `is_ci` to false so that this works even when running Volta's CI Test Suite
    ci_info::mock_ci(&CiInfo::new());

    assert_that!(
        s.volta("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("Error details written to[..]")
    );

    let mut log_dir_contents = s.read_log_dir().expect("Could not read log directory");
    assert_that!(log_dir_contents.next(), some());
}

#[test]
fn no_error_log_if_no_underlying_cause() {
    let s = sandbox().build();

    assert_that!(
        s.volta("use"),
        execs()
            .with_status(ExitCode::InvalidArguments as i32)
            .with_stderr_does_not_contain("Error details written to[..]")
    );

    // The log directory may not exist at all. If so, we know we didn't write to it
    if let Some(mut log_dir_contents) = s.read_log_dir() {
        assert_that!(log_dir_contents.next(), none());
    }
}

#[test]
fn cause_shown_in_ci() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .env("VOLTA_LOGLEVEL", "error")
        .build();

    // Mock a CI environment so this works even when running locally
    let mut ci_mock = CiInfo::new();
    ci_mock.vendor = Some(Vendor::GitHubActions);
    ci_mock.ci = true;
    ci_info::mock_ci(&ci_mock);

    assert_that!(
        s.volta("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Error cause[..]")
    );
}

#[test]
fn no_error_log_in_ci() {
    let s = sandbox().node_available_versions(NODE_VERSION_INFO).build();

    // Mock a CI environment so this works even when running locally
    let mut ci_mock = CiInfo::new();
    ci_mock.vendor = Some(Vendor::GitHubActions);
    ci_mock.ci = true;
    ci_info::mock_ci(&ci_mock);

    assert_that!(
        s.volta("install node@10"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_does_not_contain("Error details written to[..]")
    );

    // The log directory may not exist at all. If so, we know we didn't write to it
    if let Some(mut log_dir_contents) = s.read_log_dir() {
        assert_that!(log_dir_contents.next(), none());
    }
}
