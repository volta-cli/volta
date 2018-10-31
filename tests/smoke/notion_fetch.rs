use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::sandbox;

// use notion_fail::ExitCode;

// const BASIC_PACKAGE_JSON: &'static str = r#"{
//   "name": "test-package"
// }"#;

// fn package_json_with_pinned_node(version: &str) -> String {
//     format!(
//         r#"{{
//   "name": "test-package",
//   "toolchain": {{
//     "node": "{}"
//   }}
// }}"#,
//         version
//     )
// }

// const NODE_VERSION_INFO: &'static str = r#"[
// {"version":"v9.13.2","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
// {"version":"v8.8.923","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
// {"version":"v5.1.12","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
// {"version":"v5.0.11","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
// ]"#;

#[test]
fn fetch_node() {
    // TODO: better name for this
    let s = sandbox()
        // .package_json(&package_json_with_pinned_node("1.7.19"))
        // .node_available_versions(NODE_VERSION_INFO)
        // .node_archive_mock("5.1.12")
        .build();

    assert_that!(
        s.notion("fetch node 5"),
        execs()
            .with_status(0)
            // .with_stderr_contains("something with node 5")
    );
    // TODO: more asserts about where things are installed or whatever
}

#[test]
fn fetch_yarn() {
    // TODO
    let s = sandbox()
        // .package_json(&package_json_with_pinned_node("1.7.19"))
        .build();

    assert_that!(
        s.notion("fetch yarn 1"),
        execs()
            .with_status(0)
            // .with_stdout_contains("something with yarn 1.blah")
    );
    // TODO: more asserts about where things are installed or whatever
}
