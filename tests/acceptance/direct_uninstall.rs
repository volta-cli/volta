use crate::support::sandbox::{sandbox, Sandbox};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

fn platform_with_node(node: &str) -> String {
    format!(
        r#"{{
"node": {{
  "runtime": "{}",
  "npm": null
}},
"yarn": null
}}"#,
        node
    )
}

const PKG_CONFIG_COWSAY: &str = r#"{
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

const PKG_CONFIG_TYPESCRIPT: &str = r#"{
  "name": "typescript",
  "version": "1.4.0",
  "platform": {
    "node": "11.10.1",
    "npm": "6.7.0",
    "yarn": null
  },
  "bins": [
    "tsc",
    "tsserver"
  ],
  "manager": "Npm"
}"#;

fn bin_config(name: &str, pkg: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "package": "{}",
  "version": "1.4.0",
  "platform": {{
    "node": "11.10.1",
    "npm": "6.7.0",
    "yarn": null
  }},
  "manager": "Npm"
}}"#,
        name, pkg
    )
}

#[test]
fn npm_uninstall_uses_volta_logic() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .package_config("cowsay", PKG_CONFIG_COWSAY)
        .binary_config("cowsay", &bin_config("cowsay", "cowsay"))
        .binary_config("cowthink", &bin_config("cowthink", "cowsay"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("uninstall --global cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("[..]using Volta to uninstall cowsay")
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
}

#[test]
fn npm_uninstall_supports_multiples() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .package_config("cowsay", PKG_CONFIG_COWSAY)
        .binary_config("cowsay", &bin_config("cowsay", "cowsay"))
        .binary_config("cowthink", &bin_config("cowthink", "cowsay"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .package_config("typescript", PKG_CONFIG_TYPESCRIPT)
        .binary_config("tsc", &bin_config("tsc", "typescript"))
        .binary_config("tsserver", &bin_config("tsserver", "typescript"))
        .shim("tsc")
        .shim("tsserver")
        .package_image("typescript", "1.4.0")
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("uninstall --global cowsay typescript"),
        execs()
            .with_status(0)
            .with_stdout_contains("[..]using Volta to uninstall cowsay")
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
            .with_stdout_contains("[..]using Volta to uninstall typescript")
            .with_stdout_contains("Removed executable 'tsc' installed by 'typescript'")
            .with_stdout_contains("Removed executable 'tsserver' installed by 'typescript'")
            .with_stdout_contains("[..]package 'typescript' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));

    assert!(!Sandbox::package_config_exists("typescript"));
    assert!(!Sandbox::bin_config_exists("tsc"));
    assert!(!Sandbox::bin_config_exists("tsserver"));
    assert!(!Sandbox::shim_exists("tsc"));
    assert!(!Sandbox::shim_exists("tsserver"));
    assert!(!Sandbox::package_image_exists("typescript", "1.4.0"));
}

#[test]
fn yarn_remove_uses_volta_logic() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .package_config("cowsay", PKG_CONFIG_COWSAY)
        .binary_config("cowsay", &bin_config("cowsay", "cowsay"))
        .binary_config("cowthink", &bin_config("cowthink", "cowsay"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global remove cowsay"),
        execs()
            .with_status(0)
            .with_stdout_contains("[..]using Volta to uninstall cowsay")
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
}

#[test]
fn yarn_remove_supports_multiples() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .package_config("cowsay", PKG_CONFIG_COWSAY)
        .binary_config("cowsay", &bin_config("cowsay", "cowsay"))
        .binary_config("cowthink", &bin_config("cowthink", "cowsay"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0")
        .package_config("typescript", PKG_CONFIG_TYPESCRIPT)
        .binary_config("tsc", &bin_config("tsc", "typescript"))
        .binary_config("tsserver", &bin_config("tsserver", "typescript"))
        .shim("tsc")
        .shim("tsserver")
        .package_image("typescript", "1.4.0")
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global remove cowsay typescript"),
        execs()
            .with_status(0)
            .with_stdout_contains("[..]using Volta to uninstall cowsay")
            .with_stdout_contains("Removed executable 'cowsay' installed by 'cowsay'")
            .with_stdout_contains("Removed executable 'cowthink' installed by 'cowsay'")
            .with_stdout_contains("[..]package 'cowsay' uninstalled")
            .with_stdout_contains("[..]using Volta to uninstall typescript")
            .with_stdout_contains("Removed executable 'tsc' installed by 'typescript'")
            .with_stdout_contains("Removed executable 'tsserver' installed by 'typescript'")
            .with_stdout_contains("[..]package 'typescript' uninstalled")
    );

    // check that everything is deleted
    assert!(!Sandbox::package_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowsay"));
    assert!(!Sandbox::bin_config_exists("cowthink"));
    assert!(!Sandbox::shim_exists("cowsay"));
    assert!(!Sandbox::shim_exists("cowthink"));
    assert!(!Sandbox::package_image_exists("cowsay", "1.4.0"));

    assert!(!Sandbox::package_config_exists("typescript"));
    assert!(!Sandbox::bin_config_exists("tsc"));
    assert!(!Sandbox::bin_config_exists("tsserver"));
    assert!(!Sandbox::shim_exists("tsc"));
    assert!(!Sandbox::shim_exists("tsserver"));
    assert!(!Sandbox::package_image_exists("typescript", "1.4.0"));
}
