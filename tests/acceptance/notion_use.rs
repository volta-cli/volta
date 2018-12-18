use hamcrest2::core::Matcher;
use test_support::matchers::execs;
use support::sandbox::{sandbox, DistroMetadata, NodeFixture, YarnFixture};

use notion_fail::ExitCode;

const BASIC_PACKAGE_JSON: &'static str = r#"{
  "name": "test-package"
}"#;

fn package_json_with_pinned_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node,
        npm
    )
}

fn package_json_with_pinned_node_npm_yarn(node_version: &str, npm_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "toolchain": {{
    "node": "{}",
    "npm": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, npm_version, yarn_version
    )
}

const NODE_VERSION_INFO: &'static str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v9.27.6","npm":"5.6.17","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v8.9.10","npm":"5.6.7","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]},
{"version":"v6.19.62","npm":"3.10.1066","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip"]}
]
"#;


cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 270,
                uncompressed_size: Some(0x00280000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x00280000),
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

const YARN_VERSION_INFO: &'static str = r#"[
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
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.4.159",
        compressed_size: 177,
        uncompressed_size: Some(0x00280000),
    },
    DistroMetadata {
        version: "1.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x00280000),
    },
];

#[test]
fn use_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use node 6"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 6.19.62 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("6.19.62", "3.10.1066"),
    )
}

#[test]
fn use_node_latest() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use node latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.99.1040 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("10.99.1040", "6.2.26"),
    )
}

#[test]
fn use_yarn_no_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("error: There is no pinned node version for this project")
    );

    assert_eq!(s.read_package_json(), BASIC_PACKAGE_JSON,)
}

#[test]
fn use_yarn() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "1.0.7"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.4.159 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm_yarn("1.2.3", "1.0.7", "1.4.159"),
    )
}

#[test]
fn use_yarn_latest() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "1.0.7"))
        .yarn_latest("1.2.42")
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.notion("use yarn latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.2.42 in package.json")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm_yarn("1.2.3", "1.0.7", "1.2.42"),
    )
}

#[test]
fn use_yarn_incomplete_release() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "1.0.7"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    // Yarn 1.3.1 was an incomplete release with no released tarball.
    assert_that!(
        s.notion("use yarn 1.3.1"),
        execs()
            .with_status(4)
            .with_stderr_contains("error: No Yarn version found for = 1.3.1")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("1.2.3", "1.0.7"),
    )
}
