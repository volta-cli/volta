use std::thread;

use crate::support::temp_project::temp_project;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn install_node() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@12.16.2"), execs().with_status(0));

    assert_that!(
        p.node("--version"),
        execs().with_status(0).with_stdout_contains("v12.16.2")
    );

    assert!(p.node_version_is_fetched("12.16.2"));
    assert!(p.node_version_is_unpacked("12.16.2"));
    p.assert_node_version_is_installed("12.16.2");
}

#[test]
fn install_node_lts() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@lts"), execs().with_status(0));

    assert_that!(p.node("--version"), execs().with_status(0));
}

#[test]
fn install_node_concurrent() {
    let p = temp_project().build();

    let install = p.volta("install node@12.14.0");
    let run = p.node("--version");

    let concurrent_thread = thread::spawn(move || {
        assert_that!(install, execs().with_status(0));
        assert_that!(run, execs().with_status(0));
    });

    assert_that!(p.volta("install node@12.14.0"), execs().with_status(0));
    assert_that!(p.node("--version"), execs().with_status(0));

    assert!(concurrent_thread.join().is_ok());
}

#[test]
fn install_yarn() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@12.16.3"), execs().with_status(0));
    assert_that!(p.volta("install yarn@1.22.1"), execs().with_status(0));

    assert_that!(
        p.yarn("--version"),
        execs().with_status(0).with_stdout_contains("1.22.1")
    );

    assert!(p.yarn_version_is_fetched("1.22.1"));
    assert!(p.yarn_version_is_unpacked("1.22.1"));
    p.assert_yarn_version_is_installed("1.22.1");
}

#[test]
fn install_old_yarn() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@10.21.0"), execs().with_status(0));
    // Yarn 1.9.2 is old enough that it is no longer on the first page of results from the GitHub API
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
fn install_yarn_concurrent() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@12.14.0"), execs().with_status(0));

    let install = p.volta("install yarn@1.17.0");
    let run = p.yarn("--version");

    let concurrent_thread = thread::spawn(move || {
        assert_that!(install, execs().with_status(0));
        assert_that!(run, execs().with_status(0));
    });

    assert_that!(p.volta("install yarn@1.17.0"), execs().with_status(0));
    assert_that!(p.yarn("--version"), execs().with_status(0));

    assert!(concurrent_thread.join().is_ok());
}

#[test]
fn install_npm() {
    let p = temp_project().build();

    // node 13.10.0 is bundled with npm 6.13.7
    assert_that!(p.volta("install node@13.10.0"), execs().with_status(0));
    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.13.7")
    );

    // install npm 6.8.0 and verify that is installed correctly
    assert_that!(p.volta("install npm@6.14.3"), execs().with_status(0));
    assert!(p.npm_version_is_fetched("6.14.3"));
    assert!(p.npm_version_is_unpacked("6.14.3"));
    p.assert_npm_version_is_installed("6.14.3");

    assert_that!(
        p.npm("--version"),
        execs().with_status(0).with_stdout_contains("6.14.3")
    );
}

#[test]
fn install_npm_concurrent() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@12.14.0"), execs().with_status(0));

    let install = p.volta("install npm@6.14.2");
    let run = p.npm("--version");

    let concurrent_thread = thread::spawn(move || {
        assert_that!(install, execs().with_status(0));
        assert_that!(run, execs().with_status(0));
    });

    assert_that!(p.volta("install npm@6.14.2"), execs().with_status(0));
    assert_that!(p.npm("--version"), execs().with_status(0));

    assert!(concurrent_thread.join().is_ok());
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
    assert_that!(p.volta("install node@12.11.1"), execs().with_status(0));

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
fn install_package_concurrent() {
    let p = temp_project().build();

    assert_that!(p.volta("install node@12.14.0"), execs().with_status(0));

    let install = p.volta("install cowsay@1.3.0");
    let run = p.exec_shim("cowsay", "hello");

    let concurrent_thread = thread::spawn(move || {
        assert_that!(install, execs().with_status(0));
        assert_that!(run, execs().with_status(0));
    });

    assert_that!(p.volta("install cowsay@1.3.0"), execs().with_status(0));
    assert_that!(p.exec_shim("cowsay", "hello"), execs().with_status(0));

    assert!(concurrent_thread.join().is_ok());
}

#[test]
fn install_scoped_package() {
    let p = temp_project().build();

    // have to install node first, because we need npm
    assert_that!(p.volta("install node@12.14.1"), execs().with_status(0));

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
    assert_that!(p.volta("install node@12.12.0"), execs().with_status(0));

    assert_that!(p.volta("install elm@elm0.19.0"), execs().with_status(0));
    assert!(p.shim_exists("elm"));

    assert_that!(
        p.exec_shim("elm", "--version"),
        execs().with_status(0).with_stdout_contains("0.19.0")
    );
}
