use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::temp_project::temp_project;

// ISSUE(#208) - explicitly including the npm version will not be necessary after that
fn package_json_with_pinned_node_npm(version: &str, npm_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        version, npm_version
    )
}

fn package_json_with_pinned_node_npm_yarn(node_version: &str, npm_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, npm_version, yarn_version
    )
}

#[test]
fn autodownload_node() {
    let p = temp_project()
        .package_json(&package_json_with_pinned_node_npm("10.11.0", "6.4.1"))
        .build();

    assert_that!(
        p.node("--version"),
        execs()
            .with_status(0)
            .with_stdout_contains("v10.11.0")
    );
}

#[test]
fn autodownload_yarn() {
    let p = temp_project()
        .package_json(&package_json_with_pinned_node_npm_yarn("10.11.0", "6.4.1", "1.10.1"))
        .with_current_path()
        .build();

    assert_that!(
        p.yarn("--version"),
        execs()
            .with_status(0)
            .with_stdout_contains("1.10.1")
    );
}
