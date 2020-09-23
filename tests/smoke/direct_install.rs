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
        p.npm("install --global typescript@3.9.4 yarn@1.16.0"),
        execs().with_status(0)
    );

    assert!(p.shim_exists("tsc"));
    assert!(p.shim_exists("tsserver"));
    assert!(p.package_is_installed("typescript"));
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 3.9.4")
    );

    assert!(p.yarn_version_is_fetched("1.16.0"));
    assert!(p.yarn_version_is_unpacked("1.16.0"));
    p.assert_yarn_version_is_installed("1.16.0");

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("1.16.0")
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
        p.yarn("global add typescript@4.0.2 npm@6.4.0"),
        execs().with_status(0)
    );

    assert!(p.shim_exists("tsc"));
    assert!(p.shim_exists("tsserver"));
    assert!(p.package_is_installed("typescript"));
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 4.0.2")
    );

    assert!(p.npm_version_is_fetched("6.4.0"));
    assert!(p.npm_version_is_unpacked("6.4.0"));
    p.assert_npm_version_is_installed("6.4.0");

    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.4.0")
    );
}
