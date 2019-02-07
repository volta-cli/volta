use crate::support::temp_project::temp_project;

use hamcrest2::{assert_that, core::Matcher};
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
