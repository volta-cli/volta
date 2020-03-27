use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

static PACKAGE_JSON_WITH_PINNED_NODE: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "12.13.0"
    }
}"#;

static PACKAGE_JSON_WITH_PINNED_NODE_NPM: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "12.13.0",
        "npm": "6.13.4"
    }
}"#;

static PACKAGE_JSON_WITH_PINNED_NODE_YARN: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "12.13.0",
        "yarn": "1.22.0"
    }
}"#;

#[test]
fn autodownload_node() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE)
        .build();

    assert_that!(
        p.node("--version"),
        execs().with_status(0).with_stdout_contains("v12.13.0")
    );
}

#[test]
fn autodownload_npm() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE_NPM)
        .build();

    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.13.4")
    );
}

#[test]
fn autodownload_yarn() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE_YARN)
        .build();

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("1.22.0")
    );
}
