use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::env::UNSAFE_GLOBAL;
use volta_fail::ExitCode;

const PACKAGE_JSON: &'static str = r#"{
    "name": "text-package",
    "toolchain": {
        "node": "10.22.123",
        "yarn": "4.55.633"
    }
}"#;

#[test]
fn npm_prevents_global_install() {
    let s = sandbox().package_json(PACKAGE_JSON).build();

    assert_that!(
        s.npm("install ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("i ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("install ember-cli -g"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("i -g ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("-g i ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("add ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("isntall --global ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );
}

#[test]
fn npm_allows_global_install_with_env_variable() {
    let s = sandbox()
        .package_json(PACKAGE_JSON)
        .env(UNSAFE_GLOBAL, "1")
        .build();

    // Since we are using a fake Node version, we expect to get an error about being unable to download
    assert_that!(
        s.npm("i -g ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("[..]Global package installs are not recommended.")
            .with_stderr_contains("[..]Could not download node version[..]")
    );
}

#[test]
fn yarn_prevents_global_add() {
    let s = sandbox().package_json(PACKAGE_JSON).build();

    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.yarn("--verbose global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );

    assert_that!(
        s.yarn("global --verbose add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Global package installs are not recommended.")
    );
}

#[test]
fn yarn_allows_global_add_with_env_variable() {
    let s = sandbox()
        .package_json(PACKAGE_JSON)
        .env(UNSAFE_GLOBAL, "1")
        .build();

    // Since we are using a fake Yarn/Node version, we expect to get an error about being unable to download
    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("[..]Global package installs are not recommended.")
            .with_stderr_contains("[..]Could not download node version[..]")
    );
}
