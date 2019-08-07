//! Define the "plain" format style for list commands.

use textwrap::Wrapper;

use volta_core::style::text_width;

use super::{Node, Package, PackageManager, Source, Toolchain};

static INDENTATION: &'static str = "    ";
static NO_RUNTIME: &'static str = "⚡️ No Node runtimes installed!

You can install a runtime by running `volta install node`. See `volta help install` for
details and more options.";

pub(super) fn format(toolchain: &Toolchain) -> Option<String> {
    // Formatting here depends on the toolchain: we do different degrees of
    // indentation
    Some(match toolchain {
        Toolchain::Node(runtimes) => display_node(&runtimes),
        Toolchain::Active {
            runtime,
            package_manager,
            packages,
        } => display_active(runtime, package_manager, packages),
        Toolchain::All {
            runtimes,
            package_managers,
            packages,
        } => display_all(runtimes, package_managers, packages),
        Toolchain::PackageManagers(package_managers) => display_package_managers(package_managers),
        Toolchain::Packages(packages) => display_packages(packages),
        Toolchain::Tool {
            name,
            host_packages,
        } => display_tool(name, host_packages),
    })
}

/// Format the output for `Toolchain::Active`.
///
/// Accepts the components *from* the toolchain rather than the item itself so
/// that
fn display_active(
    node: &Option<Node>,
    package_manager: &Option<PackageManager>,
    packages: &[Package],
) -> String {
    match (node, package_manager, packages) {
        (None, _, _) => NO_RUNTIME.to_string(),
        (Some(node), Some(package_manager), packages) => unimplemented!(),
        _ => unimplemented!(),
    }
}

/// Format the output for `Toolchain::All`.
fn display_all(
    runtimes: &[Node],
    package_managers: &[PackageManager],
    packages: &[Package],
) -> String {
    unimplemented!()
}

/// Format the output for `Toolchain::Node`.
///
/// Accepts only `runtimes` since this printer *should* be ignorant of all other
/// types of `Toolchain` items.
fn display_node(runtimes: &[Node]) -> String {
    if runtimes.is_empty() {
        NO_RUNTIME.to_string()
    } else {
        let width = text_width().unwrap_or(0);
        let versions = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &runtimes
                    .iter()
                    .map(|runtime| format!("v{}{}", runtime.version, runtime.source))
                    .collect::<Vec<String>>()
                    .join("\n"),
            );

        format!("⚡️ Node runtimes in your toolchain:\n\n{}", versions)
    }
}

/// Format the output for `Toolchain::PackageManager`.
fn display_package_managers(package_managers: &[PackageManager]) -> String {
    unimplemented!()
}

/// Format a set of `Toolchain::Package`s and their associated tools.
fn display_packages(packages: &[Package]) -> String {
    unimplemented!()
}

/// Format the output for a specific tool from a set of `Toolchain::Package`s.
fn display_tool(tool: &str, host_packages: &[Package]) -> String {
    unimplemented!()
}

// These tests are organized by way of the *commands* supplied to `list`, unlike
// in the `plain` module, because the formatting varies by command here, as it
// does not there.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use semver::Version;

    use super::*;

    lazy_static! {
        static ref NODE_12: Version = Version::from((12, 2, 0));
        static ref NODE_11: Version = Version::from((11, 9, 0));
        static ref NODE_10: Version = Version::from((10, 15, 3));
        static ref YARN_VERSION: Version = Version::from((1, 16, 0));
        static ref PROJECT_PATH: PathBuf = PathBuf::from("~/path/to/project.json");
    }

    mod active {
        use super::*;
        use crate::command::list::{
            human::display_active, Node, PackageManager, PackageManagerKind, Source,
        };

        #[test]
        fn no_runtimes() {
            let runtime = None;
            let package_manager = None;
            let packages = vec![];
            assert_eq!(
                display_active(&runtime, &package_manager, &packages).as_str(),
                NO_RUNTIME
            );
        }

        #[test]
        fn runtime_only_user() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (default)

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::User,
                version: NODE_12.clone(),
            });
            let package_manager = None;
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn runtime_only_project() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (project @ ~/path/to/project.json)
    npm: built-in
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            });
            let package_manager = None;
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn runtime_and_yarn_user() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (default)
    Yarn: v1.16.0 (default)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::User,
                version: NODE_12.clone(),
            });
            let package_manager = Some(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::User,
                version: YARN_VERSION.clone(),
            });
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn runtime_and_yarn_mixed() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (default)
    Yarn: v1.16.0 (current @ ~/path/to/project.json)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::User,
                version: NODE_12.clone(),
            });
            let package_manager = Some(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            });
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn runtime_and_yarn_project() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (current @ ~/path/to/project.json)
    Yarn: v1.16.0 (current @ ~/path/to/project.json)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            });
            let package_manager = Some(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            });
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn with_user_tools() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (current @ ~/path/to/project.json)
    Yarn: v1.16.0 (current @ ~/path/to/project.json)
    Tool binaries available:
        create-react-app, tsc, tsserver (default)

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            });
            let package_manager = Some(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            });
            let packages = vec![
                Package {
                    name: "create-react-app".to_string(),
                    source: Source::User,
                    version: Version::from((3, 0, 1)),
                    node: NODE_12.clone(),
                    tools: vec!["create-react-app".to_string()],
                },
                Package {
                    name: "typescript".to_string(),
                    source: Source::User,
                    version: Version::from((3, 4, 3)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
            ];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn with_project_tools() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (current @ ~/path/to/project.json)
    Yarn: v1.16.0 (current @ ~/path/to/project.json)
    Tool binaries available:
        tsc, tsserver (default)
        create-react-app (current @ ~/path-to-project.json)

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            });
            let package_manager = Some(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            });
            let packages = vec![
                Package {
                    name: "create-react-app".to_string(),
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: Version::from((3, 0, 1)),
                    node: NODE_12.clone(),
                    tools: vec!["create-react-app".to_string()],
                },
                Package {
                    name: "typescript".to_string(),
                    source: Source::User,
                    version: Version::from((3, 4, 3)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
            ];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }
    }

    mod node {
        use std::path::PathBuf;

        use semver::Version;

        use super::super::*;
        use super::*;

        #[test]
        fn no_runtimes() {
            let expected = NO_RUNTIME;

            let runtimes = [];
            assert_eq!(display_node(&runtimes).as_str(), expected);
        }

        #[test]
        fn single_default() {
            let expected = "⚡️ Node runtimes in your toolchain:

    v10.15.3 (default)";
            let runtimes = [Node {
                source: Source::User,
                version: NODE_10.clone(),
            }];

            assert_eq!(display_node(&runtimes).as_str(), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ Node runtimes in your toolchain:

    v12.2.0 (current @ ~/path/to/project.json)";

            let runtimes = [Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            }];

            assert_eq!(display_node(&runtimes).as_str(), expected);
        }

        #[test]
        fn single_installed() {
            let expected = "⚡️ Node runtimes in your toolchain:

    v11.9.0";

            let runtimes = [Node {
                source: Source::None,
                version: NODE_11.clone(),
            }];

            assert_eq!(display_node(&runtimes).as_str(), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ Node runtimes in your toolchain:

    v12.2.0 (current @ ~/path/to/project.json)
    v11.9.0
    v10.15.3 (default)";

            let runtimes = [
                Node {
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: NODE_12.clone(),
                },
                Node {
                    source: Source::None,
                    version: NODE_11.clone(),
                },
                Node {
                    source: Source::User,
                    version: NODE_10.clone(),
                },
            ];

            assert_eq!(display_node(&runtimes), expected);
        }
    }

    mod package_managers {
        use super::*;
        use crate::command::list::Subcommand;
        use crate::command::list::{PackageManager, PackageManagerKind, Source};

        #[test]
        fn none_installed() {
            let expected = "⚡️ No <npm|Yarn> versions installed.

You can install a Yarn version by running `volta install yarn`.
See `volta help install` for details and more options.";

            assert_eq!(display_package_managers(&[]), expected);
        }

        #[test]
        fn single_default() {
            let expected = "⚡️ Yarn versions in your toolchain:

    v1.16.0 (default)";

            let package_managers = [PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::User,
                version: YARN_VERSION.clone(),
            }];

            assert_eq!(display_package_managers(&package_managers), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ Yarn versions in your toolchain:

    v1.16.0 (current @ ~/path/to/project.json)";

            let package_managers = [PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            }];

            assert_eq!(display_package_managers(&package_managers), expected);
        }

        #[test]
        fn single_installed() {
            let expected = "⚡️ Yarn versions in your toolchain:

    v1.16.0";

            let yarns = [PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::None,
                version: YARN_VERSION.clone(),
            }];

            assert_eq!(display_package_managers(&yarns), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ Yarn versions in your toolchain:

    v1.17.0 (current @ ~/path/to/project.json)
    v1.16.0 (default)
    v1.3.0";

            let yarns = [
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::None,
                    version: Version::from((1, 3, 0)),
                },
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::User,
                    version: YARN_VERSION.clone(),
                },
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: Version::from((1, 17, 0)),
                },
            ];

            assert_eq!(display_package_managers(&yarns), expected);
        }
    }

    mod packages {
        use super::*;
        use crate::command::list::{Package, Source};
        use semver::{Identifier, Version};

        #[test]
        fn none() {
            let expected = "⚡️ No tools or packages named `ember` installed.
            
You can safely install packages by running `volta install <package name>`.
See `volta help install` for details and more options.";

            assert_eq!(display_packages(&[]), expected);
        }

        #[test]
        fn single_default() {
            let expected = "⚡️ `ember` package versions in your toolchain:
            
    ember-cli@3.10.1 (default)
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::User,
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ `ember` package versions in your toolchain:
            
    ember-cli@3.10.1 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::Project(PROJECT_PATH.clone()),
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn single_fetched() {
            let expected = "⚡️ tool `ember` exists in one package on your system:
            
    ember-cli@3.10.1
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

To make it available to execute, run `volta install ember-cli@3.10.1`.
See `volta help install` for details and more options.";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::None,
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn multi_fetched() {
            let expected = "⚡️ tool `ember` exists in the following packages on your system:
            
    ember-cli@3.10.1
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

    ember-cli@3.8.2
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

To make it available to execute, run `volta install ember-cli@<version>`.
See `volta help install` for details and more options.";

            let packages = [
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ `ember` package versions in your toolchain:

    ember-cli@3.11.0-beta.3 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm
            
    ember-cli@3.10.1
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

    ember-cli@3.8.2 (default)
        binary tools: ember
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                    source: Source::User,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version {
                        major: 3,
                        minor: 11,
                        patch: 0,
                        pre: vec![Identifier::AlphaNumeric("-beta.3".to_string())],
                        build: vec![],
                    },
                    source: Source::Project(PROJECT_PATH.clone()),
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_packages(&packages), expected);
        }
    }

    mod tools {
        use super::*;
        use crate::command::list::{Package, Source};
        use semver::{Identifier, Version};

        #[test]
        fn none() {
            let expected = "⚡️ No tools or packages named `ember` installed.
            
You can safely install packages by running `volta install <package name>`.
See `volta help install` for details and more options.";

            assert_eq!(display_tool("ember", &[]), expected);
        }

        #[test]
        fn single_default() {
            let expected = "⚡️ tool `ember` available from:
            
    ember-cli@3.10.1 (default)
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::User,
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ tool `ember` available from:
            
    ember-cli@3.10.1 (current @ ~/path/to/project.json)
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::Project(PROJECT_PATH.clone()),
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn single_fetched() {
            let expected = "⚡️ tool `ember` available from:
            
    ember-cli@3.10.1
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

To make it available to execute, run `volta install ember-cli@3.10.1`.
See `volta help install` for details and more options.";

            let packages = [Package {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
                source: Source::None,
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn multi_fetched() {
            let expected = "⚡️ tool `ember` available from:
            
    ember-cli@3.10.1
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

    ember-cli@3.8.2
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

To make it available to execute, run `volta install ember-cli@<version>`.
See `volta help install` for details and more options.";

            let packages = [
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ tool `ember` available from:

    ember-cli@3.11.0-beta.3 (current @ ~/path/to/project.json)
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm
            
    ember-cli@3.10.1
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm

    ember-cli@3.8.2 (default)
        platform:
            runtime: node@v12.2.0
            package manager: built-in npm";

            let packages = [
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                    source: Source::None,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                    source: Source::User,
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package {
                    name: "ember-cli".to_string(),
                    version: Version {
                        major: 3,
                        minor: 11,
                        patch: 0,
                        pre: vec![Identifier::AlphaNumeric("-beta.3".to_string())],
                        build: vec![],
                    },
                    source: Source::Project(PROJECT_PATH.clone()),
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_tool("ember", &packages), expected);
        }
    }

    mod all {
        use super::*;
        use crate::command::list::PackageManagerKind;
        use semver::Identifier;

        #[test]
        fn empty() {
            let runtimes = [];
            let package_managers = [];
            let packages = [];

            assert_eq!(
                display_all(&runtimes, &package_managers, &packages),
                NO_RUNTIME
            );
        }

        #[test]
        fn full() {
            let expected = "⚡️ User toolchain:

    Node runtimes:
        v12.2.0 (current @ ~/path/to/project.json)
        v11.9.0
        v10.15.3 (default)

    Package managers:
        Yarn:
            v1.17.0 (current @ ~/path/to/project.json)
            v1.16.0 (default)
            v1.4.0

    Tools:
        ember-cli:
            v3.11.0-beta.3
                binaries: ember
                platform:
                    runtime: node@12.2.0
                    package manager: built-in npm

            v3.10.1 (current @ ~/path/to/project.json):
                binaries: ember
                platform:
                    runtime: node@12.2.0
                    package manager: built-in npm

            v3.8.2 (default):
                binaries: ember
                platform:
                    runtime: node@12.2.0
                    package manager: built-in npm

        typescript:
            v3.5.1 (current @ ~/path/to/project.json):
                binaries:
                platform:
                    runtime: node@12.2.0
                    package manager: built-in npm

            v3.4.3 (default):
                binaries:
                platform:
                    runtime: node@12.2.0
                    package manager: built-in npm

            ";

            let runtimes = [
                Node {
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: NODE_12.clone(),
                },
                Node {
                    source: Source::None,
                    version: NODE_11.clone(),
                },
                Node {
                    source: Source::User,
                    version: NODE_10.clone(),
                },
            ];

            let package_managers = [
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::User,
                    version: YARN_VERSION.clone(),
                },
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: Version::from((1, 17, 0)),
                },
                PackageManager {
                    kind: PackageManagerKind::Npm,
                    source: Source::None,
                    version: Version::from((1, 4, 0)),
                },
            ];

            let packages = [
                Package {
                    name: "typescript".to_string(),
                    source: Source::User,
                    version: Version::from((3, 4, 3)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package {
                    name: "typescript".to_string(),
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: Version::from((3, 5, 1)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package {
                    name: "ember".to_string(),
                    source: Source::None,
                    version: Version {
                        major: 3,
                        minor: 11,
                        patch: 0,
                        pre: vec![Identifier::AlphaNumeric("-beta.3".to_string())],
                        build: vec![],
                    },
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package {
                    name: "ember".to_string(),
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: Version::from((3, 10, 1)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package {
                    name: "ember".to_string(),
                    source: Source::User,
                    version: Version::from((3, 8, 2)),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
            ];
        }
    }
}
