use crate::support::sandbox::{
    sandbox, DistroMetadata, NodeFixture, NpmFixture, PnpmFixture, Yarn1Fixture, YarnBerryFixture,
};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

fn package_json_with_pinned_node(node: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}"
  }}
}}"#,
        node
    )
}

fn package_json_with_pinned_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node, npm
    )
}

fn package_json_with_pinned_node_pnpm(node_version: &str, pnpm_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "pnpm": "{}"
  }}
}}"#,
        node_version, pnpm_version
    )
}

fn package_json_with_pinned_node_yarn(node_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, yarn_version
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

const PNPM_VERSION_INFO: &str = r#"
{
    "name":"pnpm",
    "dist-tags": { "latest":"7.7.1" },
    "versions": {
        "6.34.0": { "version":"6.34.0", "dist": { "shasum":"", "tarball":"" }},
        "7.7.1": { "version":"7.7.1", "dist": { "shasum":"", "tarball":"" }}
    }
}
"#;

const PNPM_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "6.34.0",
        compressed_size: 500,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "7.7.1",
        compressed_size: 518,
        uncompressed_size: Some(0x0028_0000),
    },
];

const YARN_1_VERSION_INFO: &str = r#"{
    "name":"yarn",
    "dist-tags": { "latest":"1.12.99" },
    "versions": {
        "1.2.42": { "version":"1.2.42", "dist": { "shasum":"", "tarball":"" }},
        "1.4.159": { "version":"1.4.159", "dist": { "shasum":"", "tarball":"" }},
        "1.7.71": { "version":"1.7.71", "dist": { "shasum":"", "tarball":"" }},
        "1.12.99": { "version":"1.12.99", "dist": { "shasum":"", "tarball":"" }}
    }
}"#;

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

const YARN_BERRY_VERSION_INFO: &str = r#"{
    "name":"@yarnpkg/cli-dist",
    "dist-tags": { "latest":"3.12.99" },
    "versions": {
        "2.4.159": { "version":"2.4.159", "dist": { "shasum":"", "tarball":"" }},
        "3.2.42": { "version":"3.2.42", "dist": { "shasum":"", "tarball":"" }},
        "3.7.71": { "version":"3.7.71", "dist": { "shasum":"", "tarball":"" }},
        "3.12.99": { "version":"3.12.99", "dist": { "shasum":"", "tarball":"" }}
    }
}"#;

const YARN_BERRY_VERSION_FIXTURES: [DistroMetadata; 4] = [
    DistroMetadata {
        version: "2.4.159",
        compressed_size: 177,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "3.12.99",
        compressed_size: 178,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "3.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "3.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x0028_0000),
    },
];

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

const VOLTA_LOGLEVEL: &str = "VOLTA_LOGLEVEL";

#[test]
fn command_line_node() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 node --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 10.99.1040 from command-line configuration")
    );
}

#[test]
fn inherited_node() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node("9.27.6"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run node --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Node: 9.27.6 from project configuration")
    );
}

#[test]
fn command_line_npm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 --npm 8.1.5 npm --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]npm: 8.1.5 from command-line configuration")
    );
}

#[test]
fn inherited_npm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_npm("9.27.6", "4.5.6"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 npm --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]npm: 4.5.6 from project configuration")
    );
}

#[test]
fn force_bundled_npm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_npm("9.27.6", "4.5.6"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --bundled-npm npm --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]npm: 5.6.17[..]")
    );
}

#[test]
fn command_line_yarn_1() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 --yarn 1.7.71 yarn --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Yarn: 1.7.71 from command-line configuration")
    );
    assert_that!(
        s.volta("run --node 10.99.1040 --yarn 1.7.71 yarnpkg --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Yarn: 1.7.71 from command-line configuration")
    );
}

#[test]
fn command_line_yarn_3() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .yarn_berry_available_versions(YARN_BERRY_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .distro_mocks::<YarnBerryFixture>(&YARN_BERRY_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 --yarn 3.7.71 yarn --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Yarn: 3.7.71 from command-line configuration")
    );
}

#[test]
fn inherited_yarn_1() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_yarn("10.99.1040", "1.2.42"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 yarn --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Yarn: 1.2.42 from project configuration")
    );
}

#[test]
fn inherited_yarn_3() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .yarn_berry_available_versions(YARN_BERRY_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .distro_mocks::<YarnBerryFixture>(&YARN_BERRY_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_yarn("10.99.1040", "3.2.42"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 yarn --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]Yarn: 3.2.42 from project configuration")
    );
}

#[test]
fn force_no_yarn() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_1_available_versions(YARN_1_VERSION_INFO)
        .distro_mocks::<Yarn1Fixture>(&YARN_1_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_yarn("10.99.1040", "1.2.42"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --no-yarn yarn --version"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("[..]No Yarn version found in this project.")
    );
}

#[test]
fn command_line_pnpm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .pnpm_available_versions(PNPM_VERSION_INFO)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 --pnpm 6.34.0 pnpm --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]pnpm: 6.34.0 from command-line configuration")
    );
}

#[test]
fn inherited_pnpm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .pnpm_available_versions(PNPM_VERSION_INFO)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_pnpm("10.99.1040", "7.7.1"))
        .env(VOLTA_LOGLEVEL, "debug")
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 pnpm  --version"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stderr_contains("[..]pnpm: 7.7.1 from project configuration")
    );
}

#[test]
fn force_no_pnpm() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .pnpm_available_versions(PNPM_VERSION_INFO)
        .distro_mocks::<PnpmFixture>(&PNPM_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_pnpm("10.99.1040", "7.7.1"))
        .env(VOLTA_LOGLEVEL, "debug")
        .env("VOLTA_FEATURE_PNPM", "1")
        .build();

    assert_that!(
        s.volta("run --no-pnpm pnpm --version"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("[..]No pnpm version found in this project.")
    );
}
