use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_fail::ExitCode;

fn platform_with_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
    "node": {{
        "runtime": "{}",
        "npm": "{}"
    }}
}}"#,
        node, npm
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

#[test]
fn install_node_informs_newer_npm() {
    let s = sandbox()
        .platform(&platform_with_node_npm("8.9.10", "5.6.17"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.volta("install node@10.99.1040"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]this version of Node includes npm@6.2.26, which is higher than your default version (5.6.17).")
            .with_stdout_contains("[..]`volta install npm@bundled`[..]")
    );
}

#[test]
fn install_node_with_npm_hides_bundled_version() {
    let s = sandbox()
        .platform(&platform_with_node_npm("8.9.10", "6.2.26"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.volta("install node@9.27.6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..](with npm@5.6.17)[..]")
    );
}