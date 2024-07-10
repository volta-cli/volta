//! Tests for `npm uninstall`, `npm uninstall --global`, `yarn remove`, and
//! `yarn global remove`, which we support as alternatives to `volta uninstall`
//! and which should use its logic.

use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, Sandbox, Yarn1Fixture};
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

fn platform_with_node_yarn(node: &str, yarn: &str) -> String {
    format!(
        r#"{{
"node": {{
  "runtime": "{}",
  "npm": null
}},
"yarn": "{}"
}}"#,
        node, yarn
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

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 1] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 1096,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_1_VERSION_FIXTURES: [DistroMetadata; 1] = [DistroMetadata {
    version: "1.2.42",
    compressed_size: 174,
    uncompressed_size: Some(0x0028_0000),
}];

#[test]
fn npm_uninstall_uses_volta_logic() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .package_config("cowsay", PKG_CONFIG_COWSAY)
        .binary_config("cowsay", &bin_config("cowsay", "cowsay"))
        .binary_config("cowthink", &bin_config("cowthink", "cowsay"))
        .shim("cowsay")
        .shim("cowthink")
        .package_image("cowsay", "1.4.0", None)
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
    assert!(!Sandbox::package_image_exists("cowsay"));
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
        .package_image("cowsay", "1.4.0", None)
        .package_config("typescript", PKG_CONFIG_TYPESCRIPT)
        .binary_config("tsc", &bin_config("tsc", "typescript"))
        .binary_config("tsserver", &bin_config("tsserver", "typescript"))
        .shim("tsc")
        .shim("tsserver")
        .package_image("typescript", "1.4.0", None)
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
    assert!(!Sandbox::package_image_exists("cowsay"));

    assert!(!Sandbox::package_config_exists("typescript"));
    assert!(!Sandbox::bin_config_exists("tsc"));
    assert!(!Sandbox::bin_config_exists("tsserver"));
    assert!(!Sandbox::shim_exists("tsc"));
    assert!(!Sandbox::shim_exists("tsserver"));
    assert!(!Sandbox::package_image_exists("typescript"));
}

#[test]
fn npm_uninstall_without_packages_skips_volta_logic() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("uninstall -g"),
        execs()
            .with_status(0)
            .with_stdout_does_not_contain("[..]Volta is processing each package separately")
    );
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
        .package_image("cowsay", "1.4.0", None)
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
    assert!(!Sandbox::package_image_exists("cowsay"));
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
        .package_image("cowsay", "1.4.0", None)
        .package_config("typescript", PKG_CONFIG_TYPESCRIPT)
        .binary_config("tsc", &bin_config("tsc", "typescript"))
        .binary_config("tsserver", &bin_config("tsserver", "typescript"))
        .shim("tsc")
        .shim("tsserver")
        .package_image("typescript", "1.4.0", None)
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
    assert!(!Sandbox::package_image_exists("cowsay"));

    assert!(!Sandbox::package_config_exists("typescript"));
    assert!(!Sandbox::bin_config_exists("tsc"));
    assert!(!Sandbox::bin_config_exists("tsserver"));
    assert!(!Sandbox::shim_exists("tsc"));
    assert!(!Sandbox::shim_exists("tsserver"));
    assert!(!Sandbox::package_image_exists("typescript"));
}

#[test]
fn yarn_remove_without_packages_skips_volta_logic() {
    let s = sandbox()
        .platform(&platform_with_node_yarn("10.99.1040", "1.2.42"))
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global remove"),
        execs()
            .with_status(0)
            .with_stdout_does_not_contain("[..]Volta is processing each package separately")
    );
}
