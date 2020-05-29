use std::path::PathBuf;

use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;
use volta_core::error::ExitCode;

const WORKSPACE_PACKAGE_JSON: &str = r#"
{
    "volta": {
        "node": "10.11.12"
    }
}"#;

const PROJECT_PACKAGE_JSON: &str = r#"
{
    "volta": {
        "extends": "./workspace/package.json"
    }
}"#;

fn default_hooks_json() -> String {
    format!(
        r#"
{{
    "node": {{
        "distro": {{
            "template": "{}/hook/default/node/{{{{version}}}}"
        }}
    }},
    "npm": {{
        "distro": {{
            "template": "{0}/hook/default/npm/{{{{version}}}}"
        }}
    }},
    "yarn": {{
        "distro": {{
            "template": "{0}/hook/default/yarn/{{{{version}}}}"
        }}
    }}
}}"#,
        mockito::SERVER_URL
    )
}

fn project_hooks_json() -> String {
    format!(
        r#"
{{
    "yarn": {{
        "distro": {{
            "template": "{0}/hook/project/yarn/{{{{version}}}}"
        }}
    }}
}}"#,
        mockito::SERVER_URL
    )
}

fn workspace_hooks_json() -> String {
    format!(
        r#"
{{
    "npm": {{
        "distro": {{
            "template": "{0}/hook/workspace/npm/{{{{version}}}}"
        }}
    }},
    "yarn": {{
        "distro": {{
            "template": "{0}/hook/workspace/yarn/{{{{version}}}}"
        }}
    }}
}}"#,
        mockito::SERVER_URL
    )
}

#[test]
fn redirects_download() {
    let s = sandbox().default_hooks(&default_hooks_json()).build();

    assert_that!(
        s.volta("install node@1.2.3"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download node@1.2.3")
            .with_stderr_contains("[..]/hook/default/node/1.2.3")
    );
}

#[test]
fn merges_project_and_default_hooks() {
    let local_hooks: PathBuf = [".volta", "hooks.json"].iter().collect();
    let s = sandbox()
        .package_json("{}")
        .default_hooks(&default_hooks_json())
        .project_file(&local_hooks.to_string_lossy(), &project_hooks_json())
        .build();

    // Project defines yarn hooks, so those should be used
    assert_that!(
        s.volta("install yarn@3.2.1"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download yarn@3.2.1")
            .with_stderr_contains("[..]/hook/project/yarn/3.2.1")
    );

    // Project doesn't define node hooks, so should inherit from the default
    assert_that!(
        s.volta("install node@10.12.1"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download node@10.12.1")
            .with_stderr_contains("[..]/hook/default/node/10.12.1")
    );
}

#[test]
fn merges_workspace_hooks() {
    let workspace_hooks: PathBuf = ["workspace", ".volta", "hooks.json"].iter().collect();
    let workspace_package_json: PathBuf = ["workspace", "package.json"].iter().collect();
    let project_hooks: PathBuf = [".volta", "hooks.json"].iter().collect();
    let s = sandbox()
        .default_hooks(&default_hooks_json())
        .package_json(PROJECT_PACKAGE_JSON)
        .project_file(&project_hooks.to_string_lossy(), &project_hooks_json())
        .project_file(
            &workspace_package_json.to_string_lossy(),
            WORKSPACE_PACKAGE_JSON,
        )
        .project_file(&workspace_hooks.to_string_lossy(), &workspace_hooks_json())
        .build();

    // Project defines yarn hooks, so those should be used
    assert_that!(
        s.volta("pin yarn@3.1.4"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download yarn@3.1.4")
            .with_stderr_contains("[..]/hook/project/yarn/3.1.4")
    );

    // Workspace defines npm hooks, so those should be inherited
    assert_that!(
        s.volta("pin npm@5.6.7"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download npm@5.6.7")
            .with_stderr_contains("[..]/hook/workspace/npm/5.6.7")
    );

    // Neither project nor workspace defines node hooks, so should inherit from the default
    assert_that!(
        s.volta("install node@11.11.2"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download node@11.11.2")
            .with_stderr_contains("[..]/hook/default/node/11.11.2")
    );
}
