use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::sandbox;

#[test]
fn fetch_node() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("fetch node 9"),
        execs()
            .with_status(0)
    );
    // TODO: more asserts about where things are downloaded
}

#[test]
fn fetch_yarn() {
    let s = sandbox()
        .build();

    assert_that!(
        s.notion("fetch yarn 1.10"),
        execs()
            .with_status(0)
    );
    // TODO: more asserts about where things are downloaded
}
