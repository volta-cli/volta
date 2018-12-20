use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::temp_project::temp_project;

#[test]
fn fetch_node() {
    let p = temp_project()
        .build();

    assert_that!(
        p.notion("fetch node 10.4.1"),
        execs()
            .with_status(0)
    );
    // node 10.4.1 comes with npm 6.1.0
    assert_eq!(p.node_version_is_fetched("10.4.1"), true);
    assert_eq!(p.node_version_is_unpacked("10.4.1","6.1.0"), true);
}

#[test]
#[ignore] // ISSUE (#227) - This fails in CI because of the github API rate limit
fn fetch_yarn() {
    let p = temp_project()
        .build();

    assert_that!(
        p.notion("fetch yarn 1.10.1"),
        execs()
            .with_status(0)
    );
    assert_eq!(p.yarn_version_is_fetched("1.10.1"), true);
    assert_eq!(p.yarn_version_is_unpacked("1.10.1"), true);
}
