use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_fail::ExitCode;

const PACKAGE_JSON_WITH_YARN: &str = r#"{
    "name": "with-yarn",
    "volta": {
        "node": "10.22.123",
        "yarn": "4.55.633"
    }
}"#;

const PACKAGE_JSON_NO_YARN: &str = r#"{
    "name": "without-yarn",
    "volta": {
        "node": "10.22.123"
    }
}"#;

const PLATFORM_WITH_YARN: &str = r#"{
    "node":{
        "runtime":"9.11.2",
        "npm":"5.6.0"
    },
    "yarn": "1.22.300"
}"#;

const PLATFORM_NO_YARN: &str = r#"{
    "node":{
        "runtime":"9.11.2",
        "npm":"5.6.0"
    }
}"#;

#[test]
fn uses_project_yarn_if_available() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_WITH_YARN)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Using yarn@4.55.633 from project configuration")
    );
}

#[test]
fn uses_default_yarn_in_project_without_yarn() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_NO_YARN)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Using yarn@1.22.300 from default configuration")
    );
}

#[test]
fn uses_default_yarn_outside_project() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Using yarn@1.22.300 from default configuration")
    );
}

#[test]
fn throws_project_error_in_project() {
    let s = sandbox()
        .platform(PLATFORM_NO_YARN)
        .package_json(PACKAGE_JSON_NO_YARN)
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]No Yarn version found in this project.")
    );
}

#[test]
fn throws_user_error_outside_project() {
    let s = sandbox().platform(PLATFORM_NO_YARN).build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Yarn is not available.")
    );
}
