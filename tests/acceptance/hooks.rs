use std::path::PathBuf;

use crate::support::sandbox::sandbox;
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;
use volta_core::error::ExitCode;

fn default_hooks_json() -> String {
    format!(
        r#"
{{
    "node": {{
        "distro": {{
            "template": "{}/hook/default/node/{{{{version}}}}"
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
