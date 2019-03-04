use crate::support::temp_project::temp_project;

use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn fetch_node() {
    let p = temp_project().build();

    assert_that!(p.notion("fetch node 10.4.1"), execs().with_status(0));
    // node 10.4.1 comes with npm 6.1.0
    assert_eq!(p.node_version_is_fetched("10.4.1"), true);
    assert_eq!(p.node_version_is_unpacked("10.4.1", "6.1.0"), true);
}

#[test]
fn fetch_yarn() {
    let p = temp_project().build();

    assert_that!(p.notion("fetch yarn 1.10.1"), execs().with_status(0));
    assert_eq!(p.yarn_version_is_fetched("1.10.1"), true);
    assert_eq!(p.yarn_version_is_unpacked("1.10.1"), true);
}

#[test]
#[ignore]
fn fetch_npm() {
    // ISSUE(#292): Get this test working after pinning npm is correct
    let p = temp_project().build();

    assert_that!(p.notion("fetch npm 6.7.0"), execs().with_status(0));
    assert_eq!(p.npm_version_is_fetched("6.7.0"), true);
    assert_eq!(p.npm_version_is_unpacked("6.7.0"), true);
}

#[test]
fn fetch_package() {
    let p = temp_project().build();

    assert_that!(p.notion("fetch cowsay 1.4.0"), execs().with_status(0));
    assert_eq!(p.package_version_is_fetched("cowsay", "1.4.0"), true);
    assert_eq!(p.package_version_is_unpacked("cowsay", "1.4.0"), true);
}
