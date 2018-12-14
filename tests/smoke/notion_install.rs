use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::temp_project::temp_project;

#[test]
fn install_node() {
    let p = temp_project()
        .build();

    assert_that!(
        p.notion("install node 10.2.1"),
        execs()
            .with_status(0)
    );
    // node 10.2.1 comes with npm 5.6.0
    assert_eq!(p.node_version_is_fetched("10.2.1"), true);
    assert_eq!(p.node_version_is_unpacked("10.2.1","5.6.0"), true);
    assert_eq!(p.node_version_is_installed("10.2.1", "5.6.0"), true);
}

#[test]
fn install_yarn() {
    let p = temp_project()
        .build();

    assert_that!(
        p.notion("install yarn 1.9.2"),
        execs()
            .with_status(0)
    );
    assert_eq!(p.yarn_version_is_fetched("1.9.2"), true);
    assert_eq!(p.yarn_version_is_unpacked("1.9.2"), true);
    assert_eq!(p.yarn_version_is_installed("1.9.2"), true);
}
