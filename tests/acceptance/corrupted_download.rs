use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, YarnFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

const NODE_VERSION_INFO: &str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v0.0.1","npm":"0.0.2","lts": "Sure","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]}
]
"#;

const NODE_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "0.0.1",
        compressed_size: 10,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "10.99.1040",
        compressed_size: 273,
        uncompressed_size: Some(0x0028_0000),
    },
];

const YARN_VERSION_INFO: &str = r#"[
{"tag_name":"v0.0.1","assets":[{"name":"yarn-v0.0.1.tar.gz"}]},
{"tag_name":"v1.2.42","assets":[{"name":"yarn-v1.2.42.tar.gz"}]}
]"#;

const YARN_VERSION_FIXTURES: [DistroMetadata; 2] = [
    DistroMetadata {
        version: "0.0.1",
        compressed_size: 10,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x0028_0000),
    },
];

#[test]
fn install_corrupted_node_leaves_inventory_unchanged() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("install node@0.0.1"),
        execs().with_status(ExitCode::UnknownError as i32)
    );

    assert!(!s.node_inventory_archive_exists("0.0.1"));
}

#[test]
fn install_valid_node_saves_to_inventory() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("install node@10.99.1040"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert!(s.node_inventory_archive_exists("10.99.1040"));
}

#[test]
fn install_corrupted_yarn_leaves_inventory_unchanged() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("install yarn@0.0.1"),
        execs().with_status(ExitCode::UnknownError as i32)
    );

    assert!(!s.yarn_inventory_archive_exists("0.0.1"));
}

#[test]
fn install_valid_yarn_saves_to_inventory() {
    let s = sandbox()
        .node_available_versions(NODE_VERSION_INFO)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("install yarn@1.2.42"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert!(s.yarn_inventory_archive_exists("1.2.42"));
}
