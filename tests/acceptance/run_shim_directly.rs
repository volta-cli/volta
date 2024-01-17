use crate::support::sandbox::{sandbox, shim_exe};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

#[test]
fn shows_pretty_error_when_calling_shim_directly() {
    let s = sandbox().build();

    assert_that!(
        s.process(shim_exe()),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]should not be called directly[..]")
    );
}
