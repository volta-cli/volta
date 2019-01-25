use hamcrest2::core::Matcher;
use support::sandbox::{sandbox, DistroMetadata, NodeFixture, YarnFixture};
use test_support::matchers::execs;

use notion_fail::ExitCode;

cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 1096,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_VERSION_FIXTURES: [DistroMetadata; 1] = [DistroMetadata {
    version: "1.12.99",
    compressed_size: 178,
    uncompressed_size: Some(0x00280000),
}];

#[test]
fn npm_prevents_global_install() {
    let s = sandbox()
        .platform(r#"{"node":{"runtime":"10.99.1040","npm":"6.2.26"}}"#)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

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
}

#[test]
fn npm_allows_global_install_with_env_variable() {
    let s = sandbox()
        .platform(r#"{"node":{"runtime":"10.99.1040","npm":"6.2.26"}}"#)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("NOTION_ALLOW_GLOBAL", "1")
        .build();

    // Since we are using a fixture for the Node version, the execution will still fail
    // We just want to check that we didn't get the Global install error
    assert_that!(
        s.npm("i -g ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("Global package installs are not recommended.")
    );
}

#[test]
fn yarn_prevents_global_add() {
    let s = sandbox()
        .platform(r#"{"node":{"runtime":"10.99.1040","npm":"6.2.26"},"yarn":"1.12.99"}"#)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("Global package installs are not recommended.")
    );
}

#[test]
fn yarn_allows_global_add_with_env_variable() {
    let s = sandbox()
        .platform(r#"{"node":{"runtime":"10.99.1040","npm":"6.2.26"},"yarn":"1.12.99"}"#)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env("NOTION_ALLOW_GLOBAL", "1")
        .build();

    // Since we are using a fixture for the Yarn version, the execution will still fail
    // We just want to check that we didn't get the Global install error
    assert_that!(
        s.yarn("global add ember-cli"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_does_not_contain("Global package installs are not recommended.")
    );
}
