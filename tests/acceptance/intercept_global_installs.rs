use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use notion_core::env::UNSAFE_GLOBAL;
use notion_fail::ExitCode;

#[test]
fn npm_prevents_global_install() {
    let s = sandbox().build();

    assert_that!(
        s.npm("install ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("i ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("install ember-cli -g"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("i -g ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("-g i ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("add ember-cli --global"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.npm("isntall --global ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );
}

#[test]
fn npm_allows_global_install_with_env_variable() {
    let s = sandbox().env(UNSAFE_GLOBAL, "1").build();

    // Since we are using a fixture for the Node version, the execution will still fail
    // We just want to check that we didn't get the Global install error
    assert_that!(
        s.npm("i -g ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("Global package installs are not recommended.")
            .with_stderr_contains("No Node version selected.")
    );
}

#[test]
fn yarn_prevents_global_add() {
    let s = sandbox().build();

    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.yarn("--verbose global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );

    assert_that!(
        s.yarn("global --verbose add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );
}

#[test]
fn yarn_allows_global_add_with_env_variable() {
    let s = sandbox().env(UNSAFE_GLOBAL, "1").build();

    // Since we are using a fixture for the Yarn version, the execution will still fail
    // We just want to check that we didn't get the Global install error
    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("Global package installs are not recommended.")
            .with_stderr_contains("No Yarn version selected.")
    );
}
