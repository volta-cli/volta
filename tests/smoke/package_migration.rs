use crate::support::temp_project::temp_project;

use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

const LEGACY_PACKAGE_CONFIG: &str = r#"{
  "name": "cowsay",
  "version": "1.1.7",
  "platform": {
    "node": {
      "runtime": "14.18.2",
      "npm": null
    },
    "yarn": null
  },
  "bins": [
    "cowsay",
    "cowthink"
  ]
}"#;

const LEGACY_BIN_CONFIG: &str = r#"{
  "name": "cowsay",
  "package": "cowsay",
  "version": "1.1.7",
  "path": "./cli.js",
  "platform": {
    "node": {
      "runtime": "14.18.2",
      "npm": null
    },
    "yarn": null
  },
  "loader": {
    "command": "node",
    "args": []
  }
}"#;

const COWSAY_HELLO: &str = r#" _______
< hello >
 -------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#;

#[test]
fn legacy_package_upgrade() {
    let p = temp_project()
        .volta_home_file("tools/user/packages/cowsay.json", LEGACY_PACKAGE_CONFIG)
        .volta_home_file("tools/user/bins/cowsay.json", LEGACY_BIN_CONFIG)
        .volta_home_file(
            "tools/image/packages/cowsay/1.3.1/README.md",
            "Mock of installed package",
        )
        .volta_home_file("layout.v2", "")
        .build();

    assert_that!(p.volta("--version"), execs().with_status(0));

    assert!(p.package_is_installed("cowsay"));

    assert_that!(
        p.exec_shim("cowsay", "hello"),
        execs().with_status(0).with_stdout_contains(COWSAY_HELLO)
    );
}
