use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::temp_project::temp_project;

#[test]
fn fetch_node() {
    let s = temp_project()
        .build();

    assert_that!(
        s.notion("fetch node 10.4.1"),
        execs()
            .with_status(0)
    );
    assert_eq!(s.node_version_is_fetched("10.4.1"), true);
    assert_eq!(s.node_version_is_unpacked("10.4.1"), true);
}

#[test]
fn fetch_yarn() {
    let s = temp_project()
        .build();

    assert_that!(
        s.notion("fetch yarn 1.10.1"),
        execs()
            .with_status(0)
    );
    assert_eq!(s.yarn_version_is_fetched("1.10.1"), true);
    assert_eq!(s.yarn_version_is_unpacked("1.10.1"), true);
}
