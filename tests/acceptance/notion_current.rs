use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use notion_fail::ExitCode;

const BASIC_PACKAGE_JSON: &'static str = r#"{
  "name": "test-package"
}"#;

fn package_json_with_pinned_node(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node, npm
    )
}

#[test]
fn pinned_project() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("4.1.0", "2.14.3"))
        .build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("project: v4.1.0 (active)")
    );
}

#[test]
fn pinned_project_with_user_node_default() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("4.1.0", "2.14.3"))
        .platform(r#"{"node":{"runtime":"9.11.2","npm":"5.6.0"}}"#)
        .build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("project: v4.1.0 (active)")
            .with_stdout_contains("user: v9.11.2")
    );
}

#[test]
fn unpinned_project() {
    let s = sandbox().package_json(BASIC_PACKAGE_JSON).build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(ExitCode::NoVersionMatch as i32)
            .with_stderr_contains("error: no versions found")
    );
}

#[test]
fn unpinned_project_with_user_node_default() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .platform(r#"{"node":{"runtime":"9.11.2","npm":"5.6.0"}}"#)
        .build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v9.11.2 (active)")
    );
}

#[test]
fn no_project() {
    let s = sandbox().build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(ExitCode::NoVersionMatch as i32)
            .with_stderr_contains("error: no versions found")
    );
}

#[test]
fn no_project_with_user_node_default() {
    let s = sandbox()
        .platform(r#"{"node":{"runtime":"9.11.2","npm":"5.6.0"}}"#)
        .build();

    assert_that!(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v9.11.2 (active)")
    );
}
