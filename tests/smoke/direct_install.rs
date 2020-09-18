use crate::support::temp_project::temp_project;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn npm_global_install() {
    let p = temp_project().build();

    // Have to install node to ensure npm is available
    assert_that!(p.volta("install node@14.1.0"), execs().with_status(0));

    assert_that!(
        p.npm("install --global typescript@3.9.4"),
        execs().with_status(0)
    );
    assert!(p.shim_exists("tsc"));

    assert!(p.package_is_installed("typescript"));

    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 3.9.4")
    );
}

#[test]
fn yarn_global_add() {
    let p = temp_project().build();

    // Have to install node and yarn first
    assert_that!(
        p.volta("install node@14.2.0 yarn@1.22.5"),
        execs().with_status(0)
    );

    assert_that!(
        p.yarn("global add typescript@4.0.2"),
        execs().with_status(0)
    );
    assert!(p.shim_exists("tsc"));

    assert!(p.package_is_installed("typescript"));

    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 4.0.2")
    );
}
