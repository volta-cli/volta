use crate::support::sandbox::{sandbox, Sandbox};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[test]
fn empty_volta_home_is_created() {
    let s = sandbox().build();

    // clear out the .volta dir
    s.remove_volta_home();

    // VOLTA_HOME starts out non-existent, with no shims
    assert!(!Sandbox::path_exists(".volta"));
    assert!(!Sandbox::shim_exists("node"));

    // running volta triggers automatic creation
    assert_that!(s.volta("--version"), execs().with_status(0));

    // home directories should all be created
    assert!(Sandbox::path_exists(".volta"));
    assert!(Sandbox::path_exists(".volta/bin"));
    assert!(Sandbox::path_exists(".volta/cache/node"));
    assert!(Sandbox::path_exists(".volta/log"));
    assert!(Sandbox::path_exists(".volta/tmp"));
    assert!(Sandbox::path_exists(".volta/tools/image/node"));
    assert!(Sandbox::path_exists(".volta/tools/image/yarn"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/yarn"));
    assert!(Sandbox::path_exists(".volta/tools/user"));

    // Layout file should now exist
    assert!(Sandbox::path_exists(".volta/layout.v2"));

    // shims should all be created
    // NOTE: this doesn't work in Windows, because the default shims are stored separately
    #[cfg(unix)]
    {
        assert!(Sandbox::shim_exists("node"));
        assert!(Sandbox::shim_exists("yarn"));
        assert!(Sandbox::shim_exists("npm"));
        assert!(Sandbox::shim_exists("npx"));
    }
}

#[test]
fn legacy_v0_volta_home_is_upgraded() {
    let s = sandbox().build();

    // directories that are already created by the test framework
    assert!(Sandbox::path_exists(".volta"));
    assert!(Sandbox::path_exists(".volta/cache/node"));
    assert!(Sandbox::path_exists(".volta/tmp"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/yarn"));

    // Layout file is not there
    assert!(!Sandbox::path_exists(".volta/layout.v1"));
    assert!(!Sandbox::path_exists(".volta/layout.v2"));

    // running volta should not create anything else
    assert_that!(s.volta("--version"), execs().with_status(0));

    // everything should be the same as before running the command
    assert!(Sandbox::path_exists(".volta"));
    assert!(Sandbox::path_exists(".volta/cache/node"));
    assert!(Sandbox::path_exists(".volta/tmp"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/yarn"));

    // Layout file should now exist
    assert!(Sandbox::path_exists(".volta/layout.v2"));

    // shims should all be created
    // NOTE: this doesn't work in Windows, because the default shims are stored separately
    #[cfg(unix)]
    {
        assert!(Sandbox::shim_exists("node"));
        assert!(Sandbox::shim_exists("yarn"));
        assert!(Sandbox::shim_exists("npm"));
        assert!(Sandbox::shim_exists("npx"));
    }
}

#[test]
fn current_v2_volta_home_is_unchanged() {
    let s = sandbox().layout_file("v2").build();

    // directories that are already created by the test framework
    assert!(Sandbox::path_exists(".volta"));
    assert!(Sandbox::path_exists(".volta/layout.v2"));
    assert!(Sandbox::path_exists(".volta/cache/node"));
    assert!(Sandbox::path_exists(".volta/tmp"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/yarn"));

    // running volta should not create anything else
    assert_that!(s.volta("--version"), execs().with_status(0));

    // everything should be the same as before running the command
    assert!(Sandbox::path_exists(".volta"));
    assert!(Sandbox::path_exists(".volta/layout.v2"));
    assert!(Sandbox::path_exists(".volta/cache/node"));
    assert!(Sandbox::path_exists(".volta/tmp"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/node"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/packages"));
    assert!(Sandbox::path_exists(".volta/tools/inventory/yarn"));
}
