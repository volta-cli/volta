use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::sandbox;

#[test]
fn install_node() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("install node 10.2.1"),
        execs()
            .with_status(0)
    );
    assert_eq!(s.node_version_is_fetched("10.2.1"), true);
    assert_eq!(s.node_version_is_unpacked("10.2.1"), true);
    assert_eq!(s.node_version_is_installed("10.2.1"), true);
}

#[test]
fn install_yarn() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("install yarn 1.9.2"),
        execs()
            .with_status(0)
    );
    assert_eq!(s.yarn_version_is_fetched("1.9.2"), true);
    assert_eq!(s.yarn_version_is_unpacked("1.9.2"), true);
    assert_eq!(s.yarn_version_is_installed("1.9.2"), true);
}
