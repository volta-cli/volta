use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, NpmFixture, Yarn1Fixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

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

const NODE_VERSION_INFO: &str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v9.27.6","npm":"5.6.17","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v8.9.10","npm":"5.6.7","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v6.19.62","npm":"3.10.1066","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]}
]
"#;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 270,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 1096,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 1068,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 1055,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 1056,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_1_VERSION_INFO: &str = r#"[
{"tag_name":"v1.2.42","assets":[{"name":"yarn-v1.2.42.tar.gz"}]},
{"tag_name":"v1.3.1","assets":[{"name":"yarn-v1.3.1.msi"}]},
{"tag_name":"v1.4.159","assets":[{"name":"yarn-v1.4.159.tar.gz"}]},
{"tag_name":"v1.7.71","assets":[{"name":"yarn-v1.7.71.tar.gz"}]},
{"tag_name":"v1.12.99","assets":[{"name":"yarn-v1.12.99.tar.gz"}]}
]"#;

const YARN_1_VERSION_FIXTURES: [DistroMetadata; 4] = [
    DistroMetadata {
        version: "1.12.99",
        compressed_size: 178,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.4.159",
        compressed_size: 177,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x0028_0000),
    },
];

const NPM_VERSION_INFO: &str = r#"
{
    "name":"npm",
    "dist-tags": { "latest":"8.1.5" },
    "versions": {
        "1.2.3": { "version":"1.2.3", "dist": { "shasum":"", "tarball":"" }},
        "4.5.6": { "version":"4.5.6", "dist": { "shasum":"", "tarball":"" }},
        "8.1.5": { "version":"8.1.5", "dist": { "shasum":"", "tarball":"" }}
    }
}
"#;

const NPM_VERSION_FIXTURES: [DistroMetadata; 3] = [
    DistroMetadata {
        version: "1.2.3",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "4.5.6",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "8.1.5",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
];

#[test]
fn npm_global_install_node_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("6.19.62"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("i -g node@10.99.1040"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install Node")
            .with_stdout_contains("[..]installed and set node@10.99.1040[..]")
    );
}

#[test]
fn yarn_global_add_node_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("6.19.62"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global add node@9.27.6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install Node")
            .with_stdout_contains("[..]installed and set node@9.27.6[..]")
    );
}

#[test]
fn npm_global_install_npm_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("i -g npm@8.1.5"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install npm")
            .with_stdout_contains("[..]installed and set npm@8.1.5 as default")
    );
}

#[test]
fn yarn_global_add_npm_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global add npm@4.5.6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install npm")
            .with_stdout_contains("[..]installed and set npm@4.5.6 as default")
    );
}

#[test]
fn npm_global_install_yarn_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("i -g yarn@1.12.99"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install Yarn")
            .with_stdout_contains("[..]installed and set yarn@1.12.99 as default")
    );
}

#[test]
fn yarn_global_add_yarn_intercepts() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global add yarn@1.7.71"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]using Volta to install Yarn")
            .with_stdout_contains("[..]installed and set yarn@1.7.71 as default")
    );
}

#[test]
fn npm_global_install_supports_multiples() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("i -g npm@8.1.5 yarn@1.12.99"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]Volta is processing each package separately")
            .with_stdout_contains("[..]using Volta to install npm")
            .with_stdout_contains("[..]installed and set npm@8.1.5 as default")
            .with_stdout_contains("[..]using Volta to install Yarn")
            .with_stdout_contains("[..]installed and set yarn@1.12.99 as default")
    );
}

#[test]
fn npm_global_install_without_packages_is_treated_as_not_global() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.npm("i --global"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..]Volta is processing each package separately")
    );
}

#[test]
fn yarn_global_add_supports_multiples() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global add npm@8.1.5 yarn@1.12.99"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]Volta is processing each package separately")
            .with_stdout_contains("[..]using Volta to install npm")
            .with_stdout_contains("[..]installed and set npm@8.1.5 as default")
            .with_stdout_contains("[..]using Volta to install Yarn")
            .with_stdout_contains("[..]installed and set yarn@1.12.99 as default")
    );
}

#[test]
fn yarn_global_add_without_packages_is_treated_as_not_global() {
    let s = sandbox()
        .platform(&platform_with_node_yarn("10.99.1040", "1.2.42"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.yarn("global add"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..]Volta is processing each package separately")
    );
}

#[test]
fn npm_global_with_override_does_not_intercept() {
    let s = sandbox()
        .platform(&platform_with_node("10.99.1040"))
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .env("VOLTA_UNSAFE_GLOBAL", "1")
        .build();

    assert_that!(
        s.npm("install --global npm@8"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..]using Volta to install npm")
    );
}

#[test]
fn yarn_global_with_override_does_not_intercept() {
    let s = sandbox()
        .platform(&platform_with_node_yarn("10.99.1040", "1.12.99"))
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .env("VOLTA_UNSAFE_GLOBAL", "1")
        .build();

    assert_that!(
        s.yarn("global add npm@8"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..]using Volta to install npm")
    );
}
