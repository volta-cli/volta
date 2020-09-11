use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn fetch_node() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch node@12.16.0"), execs().with_status(0));
    assert!(p.node_version_is_fetched("12.16.0"));
    assert!(p.node_version_is_unpacked("12.16.0"));
}

#[test]
fn fetch_yarn() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch yarn@1.22.1"), execs().with_status(0));
    assert!(p.yarn_version_is_fetched("1.22.1"));
    assert!(p.yarn_version_is_unpacked("1.22.1"));
}

#[test]
fn fetch_npm() {
    let p = temp_project().build();

    assert_that!(p.volta("fetch npm@6.14.2"), execs().with_status(0));
    assert!(p.npm_version_is_fetched("6.14.2"));
    assert!(p.npm_version_is_unpacked("6.14.2"));
}

#[test]
#[cfg(not(feature = "package-global"))]
fn fetch_package() {
    let p = temp_project().build();

    // have to install node first, because we need npm
    assert_that!(p.volta("install node@12.16.1"), execs().with_status(0));

    assert_that!(p.volta("fetch cowsay@1.4.0"), execs().with_status(0));
    assert!(p.package_version_is_fetched("cowsay", "1.4.0"));
    assert!(p.package_version_is_unpacked("cowsay", "1.4.0"));
}
