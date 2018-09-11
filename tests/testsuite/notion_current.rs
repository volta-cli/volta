use support::hamcrest::assert_that;
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

#[test]
fn pinned_project() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.7.19"))
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("project: v1.7.19 (active)"),
    );
}

#[test]
fn pinned_project_with_user_node_env() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.7.19"))
        .env("NOTION_NODE_VERSION", "2.18.5")
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("project: v1.7.19 (active)")
            .with_stdout_contains("user: v2.18.5"),
    );
}

#[test]
fn pinned_project_with_user_node_default() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.7.19"))
        .catalog(
            r#"[node]
default = '9.12.11'
versions = [ '9.12.11' ]
"#,
        )
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("project: v1.7.19 (active)")
            .with_stdout_contains("user: v9.12.11"),
    );
}

#[test]
fn unpinned_project() {
    let s = sandbox().package_json(BASIC_PACKAGE_JSON).build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(ExitCode::NoVersionMatch as i32)
            .with_stderr("error: no versions found"),
    );
}

#[test]
fn unpinned_project_with_user_node_env() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .env("NOTION_NODE_VERSION", "2.18.5")
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v2.18.5 (active)"),
    );
}

#[test]
fn unpinned_project_with_user_node_default() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .catalog(
            r#"[node]
default = '9.12.11'
versions = [ '9.12.11' ]
"#,
        )
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v9.12.11 (active)"),
    );
}

#[test]
fn no_project() {
    let s = sandbox().build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(ExitCode::NoVersionMatch as i32)
            .with_stderr("error: no versions found"),
    );
}

#[test]
fn no_project_with_user_node_env() {
    let s = sandbox().env("NOTION_NODE_VERSION", "2.18.5").build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v2.18.5 (active)"),
    );
}

#[test]
fn no_project_with_user_node_default() {
    let s = sandbox()
        .catalog(
            r#"[node]
default = '9.12.11'
versions = [ '9.12.11' ]
"#,
        )
        .build();

    assert_that(
        s.notion("current"),
        execs()
            .with_status(0)
            .with_stdout_contains("user: v9.12.11 (active)"),
    );
}
