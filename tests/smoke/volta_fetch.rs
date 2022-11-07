use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn fetch_node() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch node@14.17.6"), execs().with_status(0));
    assert!(p.node_version_is_fetched("14.17.6"));
    assert!(p.node_version_is_unpacked("14.17.6"));
}

#[test]
fn fetch_yarn_1() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch yarn@1.22.1"), execs().with_status(0));
    assert!(p.yarn_version_is_fetched("1.22.1"));
    assert!(p.yarn_version_is_unpacked("1.22.1"));
}

#[test]
fn fetch_yarn_3() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch yarn@3.2.0"), execs().with_status(0));
    assert!(p.yarn_version_is_fetched("3.2.0"));
    assert!(p.yarn_version_is_unpacked("3.2.0"));
}

#[test]
fn fetch_npm() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch npm@8.3.1"), execs().with_status(0));
    assert!(p.npm_version_is_fetched("8.3.1"));
    assert!(p.npm_version_is_unpacked("8.3.1"));
}
