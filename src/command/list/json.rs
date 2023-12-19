//! Define the "JSON" format style for list commands.

use serde_json::to_string_pretty;

use super::{Node, Package, PackageManager, Toolchain};

pub(super) fn format(toolchain: &Toolchain) -> Option<String> {
    let (runtimes, package_managers, packages) = match toolchain {
        Toolchain::Node(runtimes) => (describe_runtimes(&runtimes), None, None),
        Toolchain::Active {
            runtime,
            package_managers,
            packages,
        } => (
            runtime
                .as_ref()
                .and_then(|r| describe_runtimes(&[(**r).clone()])),
            describe_package_managers(&package_managers),
            describe_packages(&packages),
        ),
        Toolchain::All {
            runtimes,
            package_managers,
            packages,
        } => (
            describe_runtimes(&runtimes),
            describe_package_managers(&package_managers),
            describe_packages(&packages),
        ),
        Toolchain::PackageManagers { managers, .. } => {
            (None, describe_package_managers(&managers), None)
        }
        Toolchain::Packages(packages) => (None, None, describe_packages(&packages)),
        Toolchain::Tool {
            name,
            host_packages,
        } => (None, None, Some(describe_tool_set(name, host_packages))),
    };

    match (runtimes, package_managers, packages) {
        (Some(runtimes), Some(package_managers), Some(packages)) => {
            Some(format!("{},{},{}", runtimes, package_managers, packages))
        }
        (Some(runtimes), Some(package_managers), None) => {
            Some(format!("{},{}", runtimes, package_managers))
        }
        (Some(runtimes), None, Some(packages)) => Some(format!("{},{}", runtimes, packages)),
        (Some(runtimes), None, None) => Some(format!("{}", runtimes)),
        (None, Some(package_managers), Some(packages)) => {
            Some(format!("{},{}", package_managers, packages))
        }
        (None, Some(package_managers), None) => Some(format!("{}", package_managers)),
        (None, None, Some(packages)) => Some(format!("{}", packages)),
        (None, None, None) => None,
    }
}

#[derive(serde::Serialize)]
struct Runtimes<'a> {
    runtimes: &'a [Node],
}

fn describe_runtimes(runtimes: &[Node]) -> Option<String> {
    if runtimes.is_empty() {
        None
    } else {
        Some(to_string_pretty(&Runtimes { runtimes }).unwrap())
    }
}

#[derive(serde::Serialize)]
struct PackageManagers<'a> {
    package_managers: &'a [PackageManager],
}

fn describe_package_managers(package_managers: &[PackageManager]) -> Option<String> {
    if package_managers.is_empty() {
        None
    } else {
        Some(to_string_pretty(&PackageManagers { package_managers }).unwrap())
    }
}

#[derive(serde::Serialize)]
struct Packages<'a> {
    packages: &'a [Package],
}

fn describe_packages(packages: &[Package]) -> Option<String> {
    if packages.is_empty() {
        None
    } else {
        Some(to_string_pretty(&Packages { packages }).unwrap())
    }
}

#[derive(serde::Serialize)]
struct Tool<'a> {
    name: &'a str,
    host: &'a Package,
}

fn describe_tool_set(name: &str, hosts: &[Package]) -> String {
    hosts
        .into_iter()
        .map(|host| to_string_pretty(&Tool { name, host }).unwrap())
        .collect::<Vec<String>>()
        .join("\n")
}

// These tests are organized by way of the *item* being printed, unlike in the
// `human` module, because the formatting is consistent across command formats.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use semver::Version;

    use crate::command::list::PackageDetails;

    lazy_static! {
        static ref NODE_VERSION: Version = Version::from((12, 4, 0));
        static ref TYPESCRIPT_VERSION: Version = Version::from((3, 4, 1));
        static ref YARN_VERSION: Version = Version::from((1, 16, 0));
        static ref PROJECT_PATH: PathBuf = PathBuf::from("/a/b/c");
    }

    mod package {
        use super::super::*;
        use super::*;

        #[test]
        fn single_default() {
            assert_eq!(
                describe_packages(&[Package::Default {
                    details: PackageDetails {
                        name: "typescript".into(),
                        version: TYPESCRIPT_VERSION.clone(),
                    },
                    node: NODE_VERSION.clone(),
                    tools: vec!["tsc".into(), "tsserver".into()]
                }])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "{
  \"packages\": [
    {
      \"Default\": {
        \"details\": {
          \"name\": \"typescript\",
          \"version\": \"3.4.1\"
        },
        \"node\": \"12.4.0\",
        \"tools\": [
          \"tsc\",
          \"tsserver\"
        ]
      }
    }
  ]
}"
            );
        }

        #[test]
        fn single_project() {
            assert_eq!(
                describe_packages(&[Package::Project {
                    name: "typescript".into(),
                    path: PROJECT_PATH.clone(),
                    tools: vec!["tsc".into(), "tsserver".into()]
                }])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "{
  \"packages\": [
    {
      \"Project\": {
        \"name\": \"typescript\",
        \"tools\": [
          \"tsc\",
          \"tsserver\"
        ],
        \"path\": \"/a/b/c\"
      }
    }
  ]
}"
            );
        }

        #[test]
        fn mixed() {
            assert_eq!(
                describe_packages(&[
                    Package::Project {
                        name: "typescript".into(),
                        path: PROJECT_PATH.clone(),
                        tools: vec!["tsc".into(), "tsserver".into()]
                    },
                    Package::Default {
                        details: PackageDetails {
                            name: "ember-cli".into(),
                            version: Version::from((3, 10, 0)),
                        },
                        node: NODE_VERSION.clone(),
                        tools: vec!["ember".into()],
                    },
                    Package::Fetched(PackageDetails {
                        name: "create-react-app".into(),
                        version: Version::from((1, 0, 0)),
                    })
                ])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "{
  \"packages\": [
    {
      \"Project\": {
        \"name\": \"typescript\",
        \"tools\": [
          \"tsc\",
          \"tsserver\"
        ],
        \"path\": \"/a/b/c\"
      }
    },
    {
      \"Default\": {
        \"details\": {
          \"name\": \"ember-cli\",
          \"version\": \"3.10.0\"
        },
        \"node\": \"12.4.0\",
        \"tools\": [
          \"ember\"
        ]
      }
    },
    {
      \"Fetched\": {
        \"name\": \"create-react-app\",
        \"version\": \"1.0.0\"
      }
    }
  ]
}"
            );
        }

        #[test]
        fn installed_not_set() {
            assert_eq!(
                describe_packages(&[Package::Fetched(PackageDetails {
                    name: "typescript".into(),
                    version: TYPESCRIPT_VERSION.clone(),
                })])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "{
  \"packages\": [
    {
      \"Fetched\": {
        \"name\": \"typescript\",
        \"version\": \"3.4.1\"
      }
    }
  ]
}"
            );
        }
    }

    mod toolchain {
        use super::super::*;
        use super::*;
        use crate::command::list::{Node, PackageManager, PackageManagerKind, Source, Toolchain};

        #[test]
        fn full() {
            assert_eq!(
                format(&Toolchain::All {
                    runtimes: vec![
                        Node {
                            source: Source::Default,
                            version: NODE_VERSION.clone()
                        },
                        Node {
                            source: Source::None,
                            version: Version::from((8, 2, 4))
                        }
                    ],
                    package_managers: vec![
                        PackageManager {
                            kind: PackageManagerKind::Yarn,
                            source: Source::Project(PROJECT_PATH.clone()),
                            version: YARN_VERSION.clone()
                        },
                        PackageManager {
                            kind: PackageManagerKind::Yarn,
                            source: Source::Default,
                            version: Version::from((1, 17, 0))
                        }
                    ],
                    packages: vec![
                        Package::Default {
                            details: PackageDetails {
                                name: "ember-cli".into(),
                                version: Version::from((3, 10, 2)),
                            },
                            node: NODE_VERSION.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package::Project {
                            name: "ember-cli".into(),
                            path: PROJECT_PATH.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package::Default {
                            details: PackageDetails {
                                name: "typescript".into(),
                                version: TYPESCRIPT_VERSION.clone(),
                            },
                            node: NODE_VERSION.clone(),
                            tools: vec!["tsc".into(), "tsserver".into()]
                        }
                    ]
                })
                .expect("`format` with a non-empty toolchain returns `Some`")
                .as_str(),
                "{
  \"runtimes\": [
    {
      \"source\": \"Default\",
      \"version\": \"12.4.0\"
    },
    {
      \"source\": \"None\",
      \"version\": \"8.2.4\"
    }
  ]
},{
  \"package_managers\": [
    {
      \"kind\": \"Yarn\",
      \"source\": {
        \"Project\": \"/a/b/c\"
      },
      \"version\": \"1.16.0\"
    },
    {
      \"kind\": \"Yarn\",
      \"source\": \"Default\",
      \"version\": \"1.17.0\"
    }
  ]
},{
  \"packages\": [
    {
      \"Default\": {
        \"details\": {
          \"name\": \"ember-cli\",
          \"version\": \"3.10.2\"
        },
        \"node\": \"12.4.0\",
        \"tools\": [
          \"ember\"
        ]
      }
    },
    {
      \"Project\": {
        \"name\": \"ember-cli\",
        \"tools\": [
          \"ember\"
        ],
        \"path\": \"/a/b/c\"
      }
    },
    {
      \"Default\": {
        \"details\": {
          \"name\": \"typescript\",
          \"version\": \"3.4.1\"
        },
        \"node\": \"12.4.0\",
        \"tools\": [
          \"tsc\",
          \"tsserver\"
        ]
      }
    }
  ]
}"
            )
        }
    }
}
