use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn uninstall_nonexistent_pkg() {
    let s = sandbox().build();
    assert_that!(
        s.notion("uninstall cowsay"),
        execs()
            .with_status(4)
            .with_stderr_contains("error: Package `cowsay` is not installed")
    );
}
