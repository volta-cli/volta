use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, YarnFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

const PACKAGE_JSON_WITH_YARN: &str = r#"{
    "name": "with-yarn",
    "volta": {
        "node": "10.99.1040",
        "yarn": "1.12.99"
    }
}"#;

const PACKAGE_JSON_NO_YARN: &str = r#"{
    "name": "without-yarn",
    "volta": {
        "node": "10.99.1040"
    }
}"#;

const PLATFORM_WITH_YARN: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":null
    },
    "yarn": "1.7.71"
}"#;

const PLATFORM_NO_YARN: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":null
    }
}"#;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 2] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 2] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 2] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 1096,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 1068,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "1.12.99",
        compressed_size: 178,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x0028_0000),
    },
];

#[test]
fn uses_project_yarn_if_available() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_WITH_YARN)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Yarn: 1.12.99 from project configuration")
    );
}

#[test]
fn uses_default_yarn_in_project_without_yarn() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_NO_YARN)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Yarn: 1.7.71 from default configuration")
    );
}

#[test]
fn uses_default_yarn_outside_project() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Yarn: 1.7.71 from default configuration")
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
fn throws_default_error_outside_project() {
    let s = sandbox().platform(PLATFORM_NO_YARN).build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Yarn is not available.")
    );
}
