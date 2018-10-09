use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::sandbox;

#[test]
#[cfg(unix)]
fn deactivate_bash() {
    let s = sandbox()
        .notion_shell("bash")
        .path_dir("/usr/bin")
        .path_dir("/usr/local/bin")
        .build();

    assert_that!(s.notion("deactivate"), execs().with_status(0));

    assert_eq!(
        s.read_postscript(),
        "export PATH='/usr/bin:/usr/local/bin'\n",
    )
}
