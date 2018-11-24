use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::{sandbox, ArchiveFixture};

use notion_fail::ExitCode;

const BASIC_PACKAGE_JSON: &'static str = r#"{
  "name": "test-package"
}"#;

fn package_json_with_pinned_node(version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}"
  }}
}}"#,
        version
    )
}

fn package_json_with_pinned_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node,
        npm
    )
}

fn package_json_with_pinned_node_yarn(node_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, yarn_version
    )
}

const NODE_VERSION_INFO: &'static str = r#"[
{"version":"v10.9.0","npm":"6.2.0","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v9.11.2","npm":"5.6.0","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v8.9.4","npm":"5.6.0","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v6.11.1","npm":"3.10.10","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
]
"#;

const NODE_VERSION_FIXTURES: [ArchiveFixture; 4] = [
    ArchiveFixture {
        version: "10.9.0",
        compressed_size: 16266798,
        uncompressed_size: 0x00daaa03,
    },
    ArchiveFixture {
        version: "9.11.2",
        compressed_size: 16153641,
        uncompressed_size: 0x0010ac03,
    },
    ArchiveFixture {
        version: "8.9.4",
        compressed_size: 16142777,
        uncompressed_size: 0x0046ac03,
    },
    ArchiveFixture {
        version: "6.11.1",
        compressed_size: 11927174,
        uncompressed_size: 0x001eb002,
    },
];

#[test]
fn use_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .node_archive_mocks(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use node 6"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 6.11.1 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("6.11.1", "3.10.10"),
    )
}

#[test]
fn use_node_latest() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .node_archive_mocks(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use node latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.9.0 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("10.9.0", "6.2.0"),
    )
}

#[test]
fn use_yarn_no_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .yarn_available_versions(r#"[ "1.0.0", "1.0.1", "1.2.0", "1.4.0", "1.9.2", "1.9.4" ]"#)
        .yarn_archive_mocks()
        .build();

    assert_that!(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("error: There is no pinned node version for this project")
    );

    assert_eq!(s.read_package_json(), BASIC_PACKAGE_JSON,)
}

#[test]
fn use_yarn() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(r#"[ "1.0.0", "1.0.1", "1.2.0", "1.4.0", "1.9.2", "1.9.4" ]"#)
        .yarn_archive_mocks()
        .build();

    assert_that!(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.4.0 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.4.0"),
    )
}

#[test]
fn use_yarn_latest() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_latest("1.2.0")
        .yarn_archive_mocks()
        .build();

    assert_that!(
        s.notion("use yarn latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.2.0 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.2.0"),
    )
}
