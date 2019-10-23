use crate::support::sandbox::{sandbox, shim_exe, Sandbox};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn empty_volta_home_is_auto_created() {
    let s = sandbox().build();

    // clear out the .volta dir
    s.remove_volta_home();

    // VOLTA_HOME starts out non-existent, with no shims
    assert!(!Sandbox::dir_exists(".volta"));
    assert!(!Sandbox::shim_exists("node"));

    // running volta triggers automatic creation
    assert_that!(s.volta("--version"), execs().with_status(0));

    // home directories should all be created
    assert!(Sandbox::dir_exists(".volta"));
    assert!(Sandbox::dir_exists(".volta/bin"));
    assert!(Sandbox::dir_exists(".volta/cache/node"));
    assert!(Sandbox::dir_exists(".volta/log"));
    assert!(Sandbox::dir_exists(".volta/tmp"));
    assert!(Sandbox::dir_exists(".volta/tools/image/node"));
    assert!(Sandbox::dir_exists(".volta/tools/image/yarn"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/yarn"));
    assert!(Sandbox::dir_exists(".volta/tools/user"));

    // shims should all be created
    // NOTE: this doesn't work in Windows, because the shim directory
    //       is stored in the Registry, and not accessible
    #[cfg(unix)]
    {
        assert!(Sandbox::shim_exists("node"));
        assert!(Sandbox::shim_exists("yarn"));
        assert!(Sandbox::shim_exists("npm"));
        assert!(Sandbox::shim_exists("npx"));
    }
}

#[test]
fn existing_volta_home_is_unchanged() {
    let s = sandbox().build();

    // directories that are already created by the test framework
    assert!(Sandbox::dir_exists(".volta"));
    assert!(Sandbox::dir_exists(".volta/cache/node"));
    assert!(Sandbox::dir_exists(".volta/tmp"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/yarn"));

    // shims do not exist
    assert!(!Sandbox::shim_exists("node"));
    assert!(!Sandbox::shim_exists("npm"));
    assert!(!Sandbox::shim_exists("yarn"));

    // running volta should not create anything else
    assert_that!(s.volta("--version"), execs().with_status(0));

    // everything should be the same as before running the command
    assert!(Sandbox::dir_exists(".volta"));
    assert!(Sandbox::dir_exists(".volta/cache/node"));
    assert!(Sandbox::dir_exists(".volta/tmp"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::dir_exists(".volta/tools/inventory/yarn"));

    assert!(!Sandbox::shim_exists("node"));
    assert!(!Sandbox::shim_exists("yarn"));
    assert!(!Sandbox::shim_exists("npm"));
    assert!(!Sandbox::shim_exists("npx"));
}
