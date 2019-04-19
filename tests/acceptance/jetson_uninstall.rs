use crate::support::sandbox::{sandbox, Sandbox};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

const PKG_CONFIG_BASIC: &'static str = r#"{
  "name": "cowsay",
  "version": "1.4.0",
  "platform": {
    "node": {
      "runtime": "11.10.1",
      "npm": "6.7.0"
    },
    "yarn": null
  },
  "bins": [
    "cowsay",
    "cowthink"
  ]
}"#;

const PKG_CONFIG_NO_BINS: &'static str = r#"{
  "name": "cowsay",
  "version": "1.4.0",
  "platform": {
    "node": {
      "runtime": "11.10.1",
      "npm": "6.7.0"
    },
    "yarn": null
  },
  "bins": []
}"#;

fn bin_config(name: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "package": "cowsay",
  "version": "1.4.0",
  "path": "./cli.js",
  "platform": {{
    "node": {{
      "runtime": "11.10.1",
      "npm": "6.7.0"
    }},
    "yarn": null
  }}
}}"#,
        name
    )
}

#[test]
fn uninstall_nonexistent_pkg() {
    // if the package doesn't exist, it should just inform the user but not throw an error
    let s = sandbox().build();

    assert_that!(
        s.jetson("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Package 'cowsay' uninstalled")
    );
}

#[test]
fn uninstall_package_basic() {
    // basic uninstall - everything exists, and everything except the cached
    // inventory files should be deleted
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .package_inventory("cowsay", "1.4.0")
        .build();

    assert_that!(
        s.jetson("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("Package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));
    // but the inventory files still exist
    assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
    assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
}

#[test]
fn uninstall_package_no_bins() {
    // the package doesn't contain any executables, it should uninstall without error
    // (normally installing a package with no executables should not happen)
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_NO_BINS)
        .package_image("cowsay", "1.4.0")
        .package_inventory("cowsay", "1.4.0")
        .build();

    assert_that!(
        s.jetson("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));
    // but the inventory files still exist
    assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
    assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
}

#[test]
fn uninstall_package_no_image() {
    // there is no unpacked & initialized package, but everything should be removed
    // (without erroring and failing to remove everything)
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .package_inventory("cowsay", "1.4.0")
        .build();

    assert_that!(
        s.jetson("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("Package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));
    // but the inventory files still exist
    assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
    assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
}

#[test]
fn uninstall_package_orphaned_bins() {
    // the package config does not exist, but for some reason there are orphaned binaries
    // those should be removed
    let s = sandbox()
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .build();

    assert_that!(
        s.jetson("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("Package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
}
