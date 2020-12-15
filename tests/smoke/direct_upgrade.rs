use crate::support::temp_project::temp_project;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;
use volta_core::error::ExitCode;

#[test]
fn npm_global_update() {
    let p = temp_project().build();

    // Install Node and typescript
    assert_that!(
        p.volta("install node@14.10.1 typescript@2.8.4"),
        execs().with_status(0)
    );
    // Confirm correct version of typescript installed
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 2.8.4")
    );

    // Update typescript
    assert_that!(p.npm("update --global typescript"), execs().with_status(0));
    // Confirm update completed successfully
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 2.9.2")
    );

    // Revert typescript update
    assert_that!(p.npm("i -g typescript@2.8.4"), execs().with_status(0));
    // Update all packages (should include typescript)
    assert_that!(p.npm("update --global"), execs().with_status(0));
    // Confirm update
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 2.9.2")
    );

    // Confirm that attempting to upgrade using `yarn` fails
    assert_that!(
        p.yarn("global upgrade typescript"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]The package 'typescript' was installed using npm.")
    );
}

#[test]
fn yarn_global_update() {
    let p = temp_project().build();

    // Install Node and Yarn
    assert_that!(
        p.volta("install node@14.10.1 yarn@1.22.5"),
        execs().with_status(0)
    );
    // Install typescript
    assert_that!(
        p.yarn("global add typescript@2.8.4"),
        execs().with_status(0)
    );
    // Confirm correct version of typescript installed
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 2.8.4")
    );

    // Upgrade typescript
    assert_that!(
        p.yarn("global upgrade typescript@2.9"),
        execs().with_status(0)
    );
    // Confirm upgrade completed successfully
    assert_that!(
        p.exec_shim("tsc", "--version"),
        execs().with_status(0).with_stdout_contains("Version 2.9.2")
    );

    // Note: Since Yarn always installs the latest version that matches your requirements and
    // 'upgrade' also gets the latest version that matches (which can change over time), an
    // immediate call to 'yarn upgrade' without packages won't result in any change.

    // This is in contrast to npm, which treats your installed version as a caret specifier when
    // runnin `npm update`

    // Confirm that attempting to upgrade using `npm` fails
    assert_that!(
        p.npm("update -g typescript"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]The package 'typescript' was installed using Yarn.")
    );
}
