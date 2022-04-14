use crate::support::temp_project::temp_project;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

const PACKAGE_JSON: &str = r#"
{
    "name": "my-library",
    "version": "1.0.0",
    "bin": {
        "mylibrary": "./index.js"
    }
}"#;

const INDEX_JS: &str = r#"#!/usr/bin/env node

console.log('VOLTA TEST');
"#;

#[test]
fn link_unlink_local_project() {
    let p = temp_project()
        .package_json(PACKAGE_JSON)
        .project_file("index.js", INDEX_JS)
        .build();

    // Install node to ensure npm is available
    assert_that!(p.volta("install node@14.15.1"), execs().with_status(0));

    // Link the current project as a global
    assert_that!(p.npm("link"), execs().with_status(0));
    // Executable should be available
    assert!(p.shim_exists("mylibrary"));
    assert!(p.package_is_installed("my-library"));
    assert_that!(
        p.exec_shim("mylibrary", ""),
        execs().with_status(0).with_stdout_contains("VOLTA TEST")
    );

    // Unlink the current project
    assert_that!(p.npm("unlink"), execs().with_status(0));
    // Executable should no longer be available
    assert!(!p.shim_exists("mylibrary"));
    assert!(!p.package_is_installed("my-library"));
}

#[test]
fn link_global_into_current_project() {
    let p = temp_project().package_json(PACKAGE_JSON).build();

    assert_that!(
        p.volta("install node@14.19.0 typescript@4.1.2"),
        execs().with_status(0)
    );

    // Link typescript into the current project
    assert_that!(p.npm("link typescript"), execs().with_status(0));
    // Typescript should now be available inside the node_modules directory
    assert!(p.project_path_exists("node_modules/typescript"));
    assert!(p.project_path_exists("node_modules/typescript/package.json"));
}
