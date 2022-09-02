use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

// Note: Node 14.11.0 is bundled with npm 6.14.8
const PACKAGE_JSON: &str = r#"{
    "name": "test-package",
    "volta": {
        "node": "14.11.0",
        "npm": "6.14.15",
        "yarn": "1.22.10"
    }
}"#;

#[test]
fn run_node() {
    let p = temp_project().build();

    assert_that!(
        p.volta("run --node 14.16.0 node --version"),
        execs().with_status(0).with_stdout_contains("v14.16.0")
    );
}

#[test]
fn run_npm() {
    let p = temp_project().build();

    assert_that!(
        p.volta("run --node 14.14.0 --npm 6.14.16 npm --version"),
        execs().with_status(0).with_stdout_contains("6.14.16")
    )
}

#[test]
fn run_yarn_1() {
    let p = temp_project().build();

    assert_that!(
        p.volta("run --node 14.16.1 --yarn 1.22.0 yarn --version"),
        execs().with_status(0).with_stdout_contains("1.22.0")
    );
}

#[test]
fn run_yarn_3() {
    let p = temp_project().build();

    assert_that!(
        p.volta("run --node 16.14.1 --yarn 3.1.1 yarn --version"),
        execs().with_status(0).with_stdout_contains("3.1.1")
    );
}

#[test]
fn inherits_project_platform() {
    let p = temp_project().package_json(PACKAGE_JSON).build();

    assert_that!(
        p.volta("run --yarn 1.21.0 yarn --version"),
        execs().with_status(0).with_stdout_contains("1.21.0")
    );
}

#[test]
fn run_environment() {
    let p = temp_project().build();

    assert_that!(
        p.volta("run --node 14.15.3 --env VOLTA_SMOKE_1234=hello node -e console.log(process.env.VOLTA_SMOKE_1234)"),
        execs().with_status(0).with_stdout_contains("hello")
    );
}
