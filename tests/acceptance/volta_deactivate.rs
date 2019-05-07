use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
#[cfg(unix)]
fn deactivate_bash() {
    let s = sandbox()
        .volta_shell("bash")
        .path_dir("/usr/bin")
        .path_dir("/usr/local/bin")
        .build();

    assert_that!(s.volta("deactivate"), execs().with_status(0));

    assert_eq!(
        s.read_postscript(),
        "export PATH='/usr/bin:/usr/local/bin'\nunset VOLTA_HOME\n",
    )
}
