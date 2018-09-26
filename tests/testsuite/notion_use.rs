use hamcrest2::core::Matcher;
use support::matchers::execs;
use support::sandbox::sandbox;

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
{"version":"v10.18.11","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v10.13.12","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v9.13.2","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v8.8.923","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
]"#;

#[test]
fn use_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .node_archive_mocks()
        .build();

    assert_that!(
        s.notion("use node 10"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.18.11 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.18.11"),
    )
}

#[test]
fn use_node_latest() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .node_archive_mocks()
        .build();

    assert_that!(
        s.notion("use node latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.18.11 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.18.11"),
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
