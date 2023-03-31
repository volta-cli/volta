use std::{thread, time};

use crate::support::events_helpers::{assert_events, match_args, match_start, match_tool_end};
use crate::support::sandbox::{
    sandbox, DistroMetadata, NodeFixture, NpmFixture, PnpmFixture, Yarn1Fixture,
};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

const PACKAGE_JSON_NODE_ONLY: &str = r#"{
    "name": "node-only",
    "volta": {
        "node": "10.99.1040"
    }
}"#;

const PACKAGE_JSON_WITH_NPM: &str = r#"{
    "name": "with-npm",
    "volta": {
        "node": "10.99.1040",
        "npm": "4.5.6"
    }
}"#;

const PACKAGE_JSON_WITH_PNPM: &str = r#"{
    "name": "with-pnpm",
    "volta": {
        "node": "10.99.1040",
        "pnpm": "7.7.1"
    }
}"#;

const PACKAGE_JSON_WITH_YARN: &str = r#"{
    "name": "with-yarn",
    "volta": {
        "node": "10.99.1040",
        "yarn": "1.12.99"
    }
}"#;

const PLATFORM_NODE_ONLY: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":null
    }
}"#;

const PLATFORM_WITH_NPM: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":"1.2.3"
    }
}"#;

const PLATFORM_WITH_PNPM: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":null
    },
    "pnpm": "7.7.1"
}"#;

const PLATFORM_WITH_YARN: &str = r#"{
    "node":{
        "runtime":"9.27.6",
        "npm":null
    },
    "yarn": "1.7.71"
}"#;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        // copy the tempfile (path in EVENTS_FILE env var) to events.json
        const EVENTS_EXECUTABLE: &str = r#"@echo off
copy %EVENTS_FILE% events.json
:: executables should clean up the temp file
del %EVENTS_FILE%
"#;
        const SCRIPT_FILENAME: &str = "write-events.bat";
        const PNPM_SHIM: &str = "pnpm.exe";
        const YARN_SHIM: &str = "yarn.exe";
    } else if #[cfg(unix)] {
        // copy the tempfile (path in EVENTS_FILE env var) to events.json
        const EVENTS_EXECUTABLE: &str = r#"#!/bin/bash
/bin/cp "$EVENTS_FILE" events.json
# executables should clean up the temp file
/bin/rm "$EVENTS_FILE"
"#;
        const SCRIPT_FILENAME: &str = "write-events.sh";
        const PNPM_SHIM: &str = "pnpm";
        const YARN_SHIM: &str = "yarn";
    } else {
        compile_error!("Unsupported platform for tests (expected 'unix' or 'windows').");
    }
}

fn events_hooks_json() -> String {
    format!(
        r#"
{{
    "events": {{
        "publish": {{
            "bin": "{}"
        }}
    }}
}}"#,
        SCRIPT_FILENAME
    )
}

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

const NPM_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "1.2.3",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "4.5.6",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
];

const PNPM_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "6.34.0",
        compressed_size: 500,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "7.7.1",
        compressed_size: 518,
        uncompressed_size: Some(0x0028_0000),
    },
];

const YARN_1_VERSION_FIXTURES: [DistroMetadata; 2] = [
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
fn uses_project_npm_if_available() {
    let s = sandbox()
        .platform(PLATFORM_WITH_NPM)
        .package_json(PACKAGE_JSON_WITH_NPM)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.npm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 10.99.1040 from project configuration")
            .with_stderr_contains("[..]npm: 4.5.6 from project configuration")
    );
}

#[test]
fn uses_bundled_npm_in_project_without_npm() {
    let s = sandbox()
        .platform(PLATFORM_WITH_NPM)
        .package_json(PACKAGE_JSON_NODE_ONLY)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.npm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 10.99.1040 from project configuration")
            .with_stderr_contains("[..]npm: 6.2.26 from project configuration")
    );
}

#[test]
fn uses_default_npm_outside_project() {
    let s = sandbox()
        .platform(PLATFORM_WITH_NPM)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.npm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 9.27.6 from default configuration")
            .with_stderr_contains("[..]npm: 1.2.3 from default configuration")
    );
}

#[test]
fn uses_bundled_npm_outside_project() {
    let s = sandbox()
        .platform(PLATFORM_NODE_ONLY)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .build();

    assert_that!(
        s.npm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 9.27.6 from default configuration")
            .with_stderr_contains("[..]npm: 5.6.17 from default configuration")
    );
}

#[test]
fn uses_project_yarn_if_available() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_WITH_YARN)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .env("VOLTA_WRITE_EVENTS_FILE", "true")
        .default_hooks(&events_hooks_json())
        .executable_file(SCRIPT_FILENAME, EVENTS_EXECUTABLE)
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]Yarn is not available.")
            .with_stderr_does_not_contain("[..]No Yarn version found in this project.")
            .with_stderr_contains("[..]Yarn: 1.12.99 from project configuration")
    );

    thread::sleep(time::Duration::from_millis(500));
    assert_events(
        &s,
        vec![
            ("tool", match_start()),
            ("yarn", match_start()),
            ("tool", match_tool_end(0)),
            (
                "args",
                match_args(format!("{} --version", YARN_SHIM).as_str()),
            ),
        ],
    );
}

#[test]
fn uses_default_yarn_in_project_without_yarn() {
    let s = sandbox()
        .platform(PLATFORM_WITH_YARN)
        .package_json(PACKAGE_JSON_NODE_ONLY)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
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
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
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
        .platform(PLATFORM_NODE_ONLY)
        .package_json(PACKAGE_JSON_NODE_ONLY)
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
    let s = sandbox().platform(PLATFORM_NODE_ONLY).build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Yarn is not available.")
    );
}

#[test]
fn uses_project_pnpm_if_available() {
    let s = sandbox()
        .platform(PLATFORM_WITH_PNPM)
        .package_json(PACKAGE_JSON_WITH_PNPM)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .env("VOLTA_WRITE_EVENTS_FILE", "true")
        .env("VOLTA_FEATURE_PNPM", "1")
        .default_hooks(&events_hooks_json())
        .executable_file(SCRIPT_FILENAME, EVENTS_EXECUTABLE)
        .build();

    assert_that!(
        s.pnpm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]pnpm is not available.")
            .with_stderr_does_not_contain("[..]No pnpm version found in this project.")
            .with_stderr_contains("[..]pnpm: 7.7.1 from project configuration")
    );

    thread::sleep(time::Duration::from_millis(500));
    assert_events(
        &s,
        vec![
            ("tool", match_start()),
            ("pnpm", match_start()),
            ("tool", match_tool_end(0)),
            (
                "args",
                match_args(format!("{} --version", PNPM_SHIM).as_str()),
            ),
        ],
    );
}

#[test]
fn uses_default_pnpm_in_project_without_pnpm() {
    let s = sandbox()
        .platform(PLATFORM_WITH_PNPM)
        .package_json(PACKAGE_JSON_NODE_ONLY)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.pnpm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]pnpm is not available.")
            .with_stderr_does_not_contain("[..]No pnpm version found in this project.")
            .with_stderr_contains("[..]pnpm: 7.7.1 from default configuration")
    );
}

#[test]
fn uses_default_pnpm_outside_project() {
    let s = sandbox()
        .platform(PLATFORM_WITH_PNPM)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "debug")
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.pnpm("--version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_does_not_contain("[..]pnpm is not available.")
            .with_stderr_does_not_contain("[..]No pnpm version found in this project.")
            .with_stderr_contains("[..]pnpm: 7.7.1 from default configuration")
    );
}

#[test]
fn uses_pnpm_throws_project_error_in_project() {
    let s = sandbox()
        .platform(PLATFORM_NODE_ONLY)
        .package_json(PACKAGE_JSON_NODE_ONLY)
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.pnpm("--version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]No pnpm version found in this project.")
    );
}
