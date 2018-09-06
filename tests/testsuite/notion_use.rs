use support::hamcrest::assert_that;
use support::matchers::execs;
use support::sandbox::sandbox;

use notion_fail::ExitCode;

#[test]
fn use_node() {
    let s = sandbox()
        .package_json(
            r#"{
  "name": "test-package"
}"#,
        )
        .build();

    assert_that(
        s.notion("use node 10"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.8.0 in package.json"),
    );

    assert_eq!(
        s.read_package_json(),
        r#"{
  "name": "test-package",
  "toolchain": {
    "node": "10.8.0"
  }
}"#.to_string()
    )
}

#[test]
fn use_node_latest() {
    let s = sandbox()
        .package_json(
            r#"{
  "name": "test-package"
}"#,
        )
        .build();

    assert_that(
        s.notion("use node latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned node to version 10.8.0 in package.json"),
    );

    assert_eq!(
        s.read_package_json(),
        r#"{
  "name": "test-package",
  "toolchain": {
    "node": "10.8.0"
  }
}"#.to_string()
    )
}

#[test]
fn use_yarn_no_node() {
    let s = sandbox()
        .package_json(
            r#"{
    "name": "test-package"
}"#,
        )
        .build();

    assert_that(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains("error: There is no pinned node version for this project"),
    );

    assert_eq!(
        s.read_package_json(),
        r#"{
    "name": "test-package"
}"#.to_string()
    )
}

#[test]
fn use_yarn() {
    let s = sandbox()
        .package_json(
            r#"{
    "name": "test-package",
    "toolchain": {
        "node": "1.2.3"
    }
}"#,
        )
        .build();

    assert_that(
        s.notion("use yarn 1.4"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.4.0 in package.json"),
    );

    assert_eq!(
        s.read_package_json(),
        r#"{
    "name": "test-package",
    "toolchain": {
        "node": "1.2.3",
        "yarn": "1.4.0"
    }
}"#.to_string()
    )
}

#[test]
fn use_yarn_latest() {
    let s = sandbox()
        .package_json(
            r#"{
    "name": "test-package",
    "toolchain": {
        "node": "1.2.3"
    }
}"#,
        )
        .build();

    assert_that(
        s.notion("use yarn latest"),
        execs()
            .with_status(0)
            .with_stdout_contains("Pinned yarn to version 1.2.0 in package.json"),
    );

    assert_eq!(
        s.read_package_json(),
        r#"{
    "name": "test-package",
    "toolchain": {
        "node": "1.2.3",
        "yarn": "1.2.0"
    }
}"#.to_string()
    )
}
