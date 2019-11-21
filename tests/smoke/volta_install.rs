use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn install_node() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@10.2.1"), execs().with_status(0));

    assert_that!(
        p.node("--version"),
        execs().with_status(0).with_stdout_contains("v10.2.1")
    );

    // node 10.2.1 comes with npm 5.6.0
    assert!(p.node_version_is_fetched("10.2.1"));
    assert!(p.node_version_is_unpacked("10.2.1", "5.6.0"));
    p.assert_node_version_is_installed("10.2.1", "5.6.0");
}

#[test]
fn install_node_lts() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@lts"), execs().with_status(0));

    assert_that!(p.node("--version"), execs().with_status(0));
}

#[test]
fn install_yarn() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@10.2.1"), execs().with_status(0));
    assert_that!(p.volta("install yarn@1.9.2"), execs().with_status(0));

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("1.9.2")
    );

    assert!(p.yarn_version_is_fetched("1.9.2"));
    assert!(p.yarn_version_is_unpacked("1.9.2"));
    p.assert_yarn_version_is_installed("1.9.2");
}

#[test]
#[ignore]
fn install_npm() {
    // ISSUE(#292): Get this test working for npm install
    let p = temp_project().build();

    // node 11.10.0 is bundled with npm 6.7.0
    assert_that!(p.volta("install node@11.10.0"), execs().with_status(0));
    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.7.0")
    );

    // install npm 6.8.0 and verify that is installed correctly
    assert_that!(p.volta("install npm@6.8.0"), execs().with_status(0));
    assert!(p.npm_version_is_fetched("6.8.0"));
    assert!(p.npm_version_is_unpacked("6.8.0"));
    p.assert_npm_version_is_installed("6.8.0");

    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.8.0")
    );
}

const COWSAY_HELLO: &'static str = r#" _______
< hello >
 -------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#;

#[test]
fn install_package() {
    let p = temp_project().build();

    // have to install node first, because we need npm
    assert_that!(p.volta("install node@10.4.1"), execs().with_status(0));

    assert_that!(p.volta("install cowsay@1.4.0"), execs().with_status(0));
    assert!(p.shim_exists("cowsay"));

    assert!(p.package_version_is_fetched("cowsay", "1.4.0"));
    assert!(p.package_version_is_unpacked("cowsay", "1.4.0"));

    assert_that!(
        p.exec_shim("cowsay", "hello"),
        execs().with_status(0).with_stdout_contains(COWSAY_HELLO)
    );
}

#[test]
fn install_scoped_package() {
    let p = temp_project().build();

    // have to install node first, because we need npm
    assert_that!(p.volta("install node@10.4.1"), execs().with_status(0));

    assert_that!(p.volta("install @wdio/cli@5.12.4"), execs().with_status(0));
    assert!(p.shim_exists("wdio"));

    assert!(p.package_version_is_fetched("@wdio/cli", "5.12.4"));
    assert!(p.package_version_is_unpacked("@wdio/cli", "5.12.4"));

    assert_that!(
        p.exec_shim("wdio", "--version"),
        execs().with_status(0).with_stdout_contains("5.12.4")
    );
}

#[test]
fn install_package_tag_version() {
    let p = temp_project().build();

    // have to install node first, because we need npm
    assert_that!(p.volta("install node@10.4.1"), execs().with_status(0));

    assert_that!(p.volta("install elm@elm0.19.0"), execs().with_status(0));
    assert!(p.shim_exists("elm"));

    assert_that!(
        p.exec_shim("elm", "--version"),
        execs().with_status(0).with_stdout_contains("0.19.0")
    );
}
