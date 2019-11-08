use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, YarnFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_fail::ExitCode;

const BASIC_PACKAGE_JSON: &'static str = r#"{
  "name": "test-package"
}"#;

fn package_json_with_pinned_node(node: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}"
  }}
}}"#,
        node
    )
}

fn package_json_with_pinned_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node, npm
    )
}

fn package_json_with_pinned_node_yarn(node_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, yarn_version
    )
}

fn package_json_with_pinned_node_npm_yarn(
    node_version: &str,
    npm_version: &str,
    yarn_version: &str,
) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "npm": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, npm_version, yarn_version
    )
}

const NODE_VERSION_INFO: &'static str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v9.27.6","npm":"5.6.17","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v8.9.10","npm":"5.6.7","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v6.19.62","npm":"3.10.1066","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
]
"#;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 270,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
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
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 1055,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 1056,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_VERSION_INFO: &'static str = r#"[
{"tag_name":"v1.2.42","assets":[{"name":"yarn-v1.2.42.tar.gz"}]},
{"tag_name":"v1.3.1","assets":[{"name":"yarn-v1.3.1.msi"}]},
{"tag_name":"v1.4.159","assets":[{"name":"yarn-v1.4.159.tar.gz"}]},
{"tag_name":"v1.7.71","assets":[{"name":"yarn-v1.7.71.tar.gz"}]},
{"tag_name":"v1.12.99","assets":[{"name":"yarn-v1.12.99.tar.gz"}]}
]"#;

const YARN_VERSION_FIXTURES: [DistroMetadata; 4] = [
    DistroMetadata {
        version: "1.12.99",
        compressed_size: 178,
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.4.159",
        compressed_size: 177,
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x00280000),
    },
];

const NPM_VERSION_INFO: &'static str = r#"
{
    "name":"npm",
    "dist-tags": { "latest":"6.8.0" },
    "versions": {
        "1.2.3": { "version":"1.2.3", "dist": { "shasum":"", "tarball":"" }},
        "4.5.6": { "version":"4.5.6", "dist": { "shasum":"", "tarball":"" }},
        "5.10.1": { "version":"5.10.1", "dist": { "shasum":"", "tarball":"" }},
        "5.10.12": { "version":"5.10.12", "dist": { "shasum":"", "tarball":"" }},
        "8.1.5": { "version":"8.1.5", "dist": { "shasum":"", "tarball":"" }}
    }
}
"#;

const VOLTA_LOGLEVEL: &'static str = "VOLTA_LOGLEVEL";

#[test]
fn pin_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("6.19.62"),
    )
}

#[test]
fn pin_node_reports_info() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]pinned node@6.19.62 (with npm@3.10.1066) in package.json")
    );
}

#[test]
fn pin_node_latest() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@latest"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.99.1040"),
    )
}

#[test]
fn pin_node_no_version() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.99.1040"),
    )
}

#[test]
fn pin_node_removes_npm() {
    // Pinning Node will set the pinned version of npm to the default for that version, so it will be omitted
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("6.19.62", "3.9.1"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@8"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("8.9.10"),
    )
}

#[test]
fn pin_yarn_no_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains(
                "[..]Cannot pin Yarn because the Node version is not pinned in this project."
            )
    );

    assert_eq!(s.read_package_json(), BASIC_PACKAGE_JSON,)
}

#[test]
fn pin_yarn() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.4.159"),
    )
}

#[test]
fn pin_yarn_reports_info() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]pinned yarn@1.4.159 in package.json")
    );
}

#[test]
fn pin_yarn_latest() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_latest("1.2.42")
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@latest"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.2.42"),
    )
}

#[test]
fn pin_yarn_no_version() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_latest("1.2.42")
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.2.42"),
    )
}

#[test]
fn pin_yarn_missing_release() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .mock_not_found()
        .build();

    assert_that!(
        s.volta("pin yarn@1.3.1"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download yarn@1.3.1")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("1.2.3"),
    )
}

#[test]
fn pin_yarn_leaves_npm() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "3.4.5"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm_yarn("1.2.3", "3.4.5", "1.4.159"),
    )
}

#[test]
#[ignore]
fn pin_npm() {
    // ISSUE(#292): Get this test working after pinning npm is correct
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .npm_available_versions(NPM_VERSION_INFO)
        .build();

    assert_that!(
        s.volta("pin npm@5.10"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("1.2.3", "5.10.12"),
    )
}

#[test]
fn pin_node_and_yarn() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@6 yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("6.19.62", "1.4.159"),
    )
}
