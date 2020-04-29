use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, NpmFixture, YarnFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_fail::ExitCode;

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

const YARN_VERSION_INFO: &str = r#"[
{"tag_name":"v1.2.42","assets":[{"name":"yarn-v1.2.42.tar.gz"}]},
{"tag_name":"v1.3.1","assets":[{"name":"yarn-v1.3.1.msi"}]},
{"tag_name":"v1.4.159","assets":[{"name":"yarn-v1.4.159.tar.gz"}]},
{"tag_name":"v1.7.71","assets":[{"name":"yarn-v1.7.71.tar.gz"}]},
{"tag_name":"v1.12.99","assets":[{"name":"yarn-v1.12.99.tar.gz"}]}
]"#;

const YARN_VERSION_FIXTURES: [DistroMetadata; 4] = [
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
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using node@10.99.1040 from command-line configuration")
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
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using node@9.27.6 from project configuration")
    );
}

#[test]
fn no_node() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("run node --version"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("[..]Node is not available.")
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
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using npm@8.1.5 from command-line configuration")
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
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using npm@4.5.6 from project configuration")
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
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using npm@5.6.17 from project configuration")
    );
}

#[test]
fn command_line_yarn() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 --yarn 1.7.71 yarn --version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using yarn@1.7.71 from command-line configuration")
    );
}

#[test]
fn inherited_yarn() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .package_json(&package_json_with_pinned_node_yarn("10.99.1040", "1.2.42"))
        .env(VOLTA_LOGLEVEL, "debug")
        .build();

    assert_that!(
        s.volta("run --node 10.99.1040 yarn --version"),
        execs()
            .with_status(ExitCode::ExecutionFailure as i32)
            .with_stderr_contains("[..]Using yarn@1.2.42 from project configuration")
    );
}

#[test]
fn force_no_yarn() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
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
