use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::temp_project::temp_project;

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

#[test]
fn autodownload_node() {
    let s = temp_project()
        .package_json(&package_json_with_pinned_node("10.11.0"))
        .build();

    assert_that!(
        s.node("--version"),
        execs()
            .with_status(0)
            .with_stdout_contains("v10.11.0")
    );
}

#[test]
fn autodownload_yarn() {
    let s = temp_project()
        .package_json(&package_json_with_pinned_node_yarn("10.11.0", "1.10.1"))
        .with_current_path()
        .build();

    assert_that!(
        s.yarn("--version"),
        execs()
            .with_status(0)
            .with_stdout_contains("1.10.1")
    );
}
