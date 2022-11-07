use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

static PACKAGE_JSON_WITH_PINNED_NODE: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "14.15.5"
    }
}"#;

static PACKAGE_JSON_WITH_PINNED_NODE_NPM: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "17.3.0",
        "npm": "8.5.1"
    }
}"#;

static PACKAGE_JSON_WITH_PINNED_NODE_YARN_1: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "16.11.1",
        "yarn": "1.22.16"
    }
}"#;

static PACKAGE_JSON_WITH_PINNED_NODE_YARN_3: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "16.14.0",
        "yarn": "3.1.0"
    }
}"#;

#[test]
fn autodownload_node() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE)
        .build();

    assert_that!(
        p.node("--version"),
        execs().with_status(0).with_stdout_contains("v14.15.5")
    );
}

#[test]
fn autodownload_npm() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE_NPM)
        .build();

    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("8.5.1")
    );
}

#[test]
fn autodownload_yarn_1() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE_YARN_1)
        .build();

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("1.22.16")
    );
}

#[test]
fn autodownload_yarn_3() {
    let p = temp_project()
        .package_json(PACKAGE_JSON_WITH_PINNED_NODE_YARN_3)
        .build();

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("3.1.0")
    );
}
