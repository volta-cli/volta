use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::sandbox;

#[test]
fn install_node() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("install node 9"),
        execs()
            .with_status(0)
    );
    // TODO: more asserts about where things are installed
}

#[test]
fn install_yarn() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("install yarn 1.10"),
        execs()
            .with_status(0)
    );
    // TODO: more asserts about where things are installed
}
