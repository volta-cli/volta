use crate::support::sandbox::{sandbox, Sandbox};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

#[cfg(feature = "package-global")]
const PKG_CONFIG_BASIC: &str = r#"{
  "name": "cowsay",
  "version": "1.4.0",
  "platform": {
    "node": "11.10.1",
    "npm": "6.7.0",
    "yarn": null
  },
  "bins": [
    "cowsay",
    "cowthink"
  ],
  "manager": "Npm"
}"#;

#[cfg(not(feature = "package-global"))]
const PKG_CONFIG_BASIC: &str = r#"{
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

#[cfg(feature = "package-global")]
const PKG_CONFIG_NO_BINS: &str = r#"{
  "name": "cowsay",
  "version": "1.4.0",
  "platform": {
    "node": "11.10.1",
    "npm": "6.7.0",
    "yarn": null
  },
  "bins": [],
  "manager": "Npm"
}"#;

#[cfg(not(feature = "package-global"))]
const PKG_CONFIG_NO_BINS: &str = r#"{
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

#[cfg(feature = "package-global")]
fn bin_config(name: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "package": "cowsay",
  "version": "1.4.0",
  "platform": {{
    "node": "11.10.1",
    "npm": "6.7.0",
    "yarn": null
  }},
  "manager": "Npm"
}}"#,
        name
    )
}

#[cfg(not(feature = "package-global"))]
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

const VOLTA_LOGLEVEL: &str = "VOLTA_LOGLEVEL";

#[test]
fn uninstall_nonexistent_pkg() {
    // if the package doesn't exist, it should just inform the user but not throw an error
    let s = sandbox().env(VOLTA_LOGLEVEL, "info").build();

    assert_that!(
        s.volta("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stderr_contains("[..]No package 'cowsay' found to uninstall")
    );
}

#[test]
fn uninstall_package_basic() {
    // basic uninstall - everything exists, and everything except the cached
    // inventory files should be deleted
    #[cfg(feature = "package-global")]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    #[cfg(not(feature = "package-global"))]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .package_inventory("cowsay", "1.4.0")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));

    #[cfg(not(feature = "package-global"))]
    {
        // but the inventory files still exist
        assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
        assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
    }
}

#[test]
fn uninstall_package_no_bins() {
    // the package doesn't contain any executables, it should uninstall without error
    // (normally installing a package with no executables should not happen)
    #[cfg(feature = "package-global")]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_NO_BINS)
        .package_image("cowsay", "1.4.0")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    #[cfg(not(feature = "package-global"))]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_NO_BINS)
        .package_image("cowsay", "1.4.0")
        .package_inventory("cowsay", "1.4.0")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));
    #[cfg(not(feature = "package-global"))]
    {
        // but the inventory files still exist
        assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
        assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
    }
}

#[test]
fn uninstall_package_no_image() {
    // there is no unpacked & initialized package, but everything should be removed
    // (without erroring and failing to remove everything)
    #[cfg(feature = "package-global")]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    #[cfg(not(feature = "package-global"))]
    let s = sandbox()
        .package_config("cowsay", PKG_CONFIG_BASIC)
        .binary_config("cowsay", &bin_config("cowsay"))
        .binary_config("cowthink", &bin_config("cowthink"))
        .shim("cowsay")
        .shim("cowthink")
        .package_inventory("cowsay", "1.4.0")
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));
    #[cfg(not(feature = "package-global"))]
    {
        // but the inventory files still exist
        assert!(Sandbox::pkg_inventory_tarball_exists("cowsay", "1.4.0"));
        assert!(Sandbox::pkg_inventory_shasum_exists("cowsay", "1.4.0"));
    }
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
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("uninstall cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
}
