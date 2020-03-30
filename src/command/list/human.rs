//! Define the "human" format style for list commands.

use textwrap::Wrapper;

use volta_core::style::{text_width, tool_version};

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
        Toolchain::PackageManagers(package_managers) => {
            format!("{}", &display_package_managers(package_managers))
        }
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
    runtime: &Option<Box<Node>>,
    package_manager: &Option<Box<PackageManager>>,
    packages: &[Package],
) -> String {
    match (runtime, package_manager, packages) {
        (None, _, _) => NO_RUNTIME.to_string(),
        (Some(runtime), Some(package_manager), packages) => {
            let width = text_width().unwrap_or(0);
            let runtime_version: String = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .fill(&format!("Node: {}", list_runtime(runtime)));
            let package_manager_version: String = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .fill(&format!("Yarn: {}", list_package_manager(package_manager)));
            let package_versions = if packages.is_empty() {
                Wrapper::new(width)
                    .initial_indent(INDENTATION)
                    .fill(&format!("Tool binaries available: NONE"))
            } else {
                Wrapper::new(width)
                    .initial_indent(INDENTATION)
                    .subsequent_indent(INDENTATION)
                    .fill(&format!(
                        "Tool binaries available:\n{}",
                        display_tool_sets(packages)
                    ))
            };

            format!(
                "⚡️ Currently active tools:\n\n{}\n{}\n{}\n\n{}",
                runtime_version,
                package_manager_version,
                package_versions,
                "See options for more detailed reports by running `volta list --help`."
            )
        }
        (Some(runtime), None, packages) => {
            let width = text_width().unwrap_or(0);
            let runtime_version: String = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .fill(&format!("node: {}", list_runtime(runtime)));
            let package_versions = if packages.is_empty() {
                Wrapper::new(width)
                    .initial_indent(INDENTATION)
                    .fill(&format!("Tool binaries available: NONE"))
            } else {
                Wrapper::new(width)
                    .initial_indent(INDENTATION)
                    .subsequent_indent(INDENTATION)
                    .fill(&format!(
                        "Tool binaries available:\n{}",
                        display_packages(packages)
                    ))
            };

            format!(
                "⚡️ Currently active tools:\n\n{}\n{}\n\n{}",
                runtime_version,
                package_versions,
                "See options for more detailed reports by running `volta list --help`."
            )
        }
    }
}

/// Format the output for `Toolchain::All`.
fn display_all(
    runtimes: &[Node],
    package_managers: &[PackageManager],
    packages: &[Package],
) -> String {
    if runtimes.is_empty() {
        NO_RUNTIME.to_string()
    } else {
        let width = text_width().unwrap_or(0);
        let runtime_versions: String = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(&format!("Node runtimes:\n{}", list_runtimes(runtimes)));
        let package_manager_versions: String = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(&INDENTATION.repeat(2))
            .fill(&format!(
                "Package managers:\n{}\n{}",
                "Yarn:",
                list_package_managers(package_managers)
            ));
        let package_versions = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(&format!("Packages:\n{}", list_packages(packages)));
        format!(
            "⚡️ User toolchain:\n\n{}\n\n{}\n\n{}",
            runtime_versions, package_manager_versions, package_versions
        )
    }
}

/// Format a set of `Toolchain::Node`s.
fn display_node(runtimes: &[Node]) -> String {
    if runtimes.is_empty() {
        NO_RUNTIME.to_string()
    } else {
        format!(
            "⚡️ Node runtimes in your toolchain:\n\n{}",
            list_runtimes(&runtimes)
        )
    }
}

/// Format a set of `Toolchain::PackageManager`s.
fn display_package_managers(package_managers: &[PackageManager]) -> String {
    if package_managers.is_empty() {
        String::from(
            "⚡️ No <npm|Yarn> versions installed.

You can install a Yarn version by running `volta install yarn`.
See `volta help install` for details and more options.",
        )
    } else {
        let width = text_width().unwrap_or(0);
        let versions = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &package_managers
                    .iter()
                    .map(list_package_manager)
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        format!("⚡️ Yarn versions in your toolchain:\n\n{}", versions)
    }
}

/// Format a set of `Toolchain::Package`s and their associated tools.
fn display_packages(packages: &[Package]) -> String {
    if packages.is_empty() {
        String::from(
            "⚡️ No tools or packages installed.

You can safely install packages by running `volta install <package name>`.
See `volta help install` for details and more options.",
        )
    } else {
        format!(
            "⚡️ Package versions in your toolchain:\n\n{}",
            list_packages(packages)
        )
    }
}

/// Format a single `Toolchain::Tool` with associated `Toolchain::Package`

fn display_tool(tool: &str, host_packages: &[Package]) -> String {
    if host_packages.is_empty() {
        format!(
            "⚡️ No tools or packages named `{}` installed.

You can safely install packages by running `volta install <package name>`.
See `volta help install` for details and more options.",
            tool
        )
    } else {
        let width = text_width().unwrap_or(0);
        let versions = Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &host_packages
                    .iter()
                    .map(list_package)
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        format!("⚡️ Tool `{}` available from:\n\n{}", tool, versions)
    }
}

/// Format a sets binaries inside `Toolchain::Package`s without detail information
fn display_tool_sets(packages: &[Package]) -> String {
    format!(
        "{}",
        &packages
            .iter()
            .map(display_tool_set)
            .collect::<Vec<String>>()
            .join("\n")
    )
}
/// Format a set binaries inside a single `Toolchain::Package` without detail information
fn display_tool_set(package: &Package) -> String {
    match package {
        Package::Default { tools, .. } | Package::Project { tools, .. } => {
            let tools = match tools.len() {
                0 => String::from(""),
                _ => format!("{}", tools.join(", ")),
            };
            let width = text_width().unwrap_or(0);

            Wrapper::new(width)
                .initial_indent(INDENTATION)
                .subsequent_indent(INDENTATION)
                .fill(&format!("{}{}", tools, list_package_source(package)))
        }
        Package::Fetched(..) => String::new(),
    }
}

/// List a set versions of `Toolchain::Node`s.
fn list_runtimes(runtimes: &[Node]) -> String {
    if runtimes.is_empty() {
        String::new()
    } else {
        let width = text_width().unwrap_or(0);
        Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &runtimes
                    .iter()
                    .map(|runtime| list_runtime(&runtime))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
    }
}

/// List a single version of `Toolchain::Node`.
fn list_runtime(runtime: &Node) -> String {
    format!("v{}{}", runtime.version, runtime.source)
}

/// List all the versions of `Toolchain::PackageManager`s.
fn list_package_managers(package_managers: &[PackageManager]) -> String {
    if package_managers.is_empty() {
        String::new()
    } else {
        let width = text_width().unwrap_or(0);
        Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &package_managers
                    .iter()
                    .map(list_package_manager)
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
    }
}

/// List a the version of `Toolchain::PackageManager`.
fn list_package_manager(package_manager: &PackageManager) -> String {
    format!("v{}{}", package_manager.version, package_manager.source)
}

/// List a set of `Toolchain::Package`s and their associated tools.
fn list_packages(packages: &[Package]) -> String {
    if packages.is_empty() {
        String::new()
    } else {
        let width = text_width().unwrap_or(0);
        Wrapper::new(width)
            .initial_indent(INDENTATION)
            .subsequent_indent(INDENTATION)
            .fill(
                &packages
                    .iter()
                    .map(list_package)
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
    }
}

/// List a single `Toolchain::Package` and its associated tools.
fn list_package(package: &Package) -> String {
    match package {
        Package::Default {
            details,
            node,
            tools,
            ..
        }
        | Package::Project {
            details,
            node,
            tools,
            ..
        } => {
            let tools = match tools.len() {
                0 => String::from(""),
                _ => format!("{}", tools.join(", ")),
            };
            let width = text_width().unwrap_or(0);

            // println!("{}\n{}\n{}\n{}", details.name, details.version, node, tools);
            let version = format!("{}{}", details.version, list_package_source(package));
            let binaries = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .subsequent_indent(INDENTATION)
                .fill(&format!("binary tools: {}", tools));
            let platform_detail = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .subsequent_indent(INDENTATION)
                .fill(&format!(
                    "runtime: {}\npackage manager: {}",
                    tool_version("node", &node),
                    // TODO: Should be updated when we support installing with custom package_managers,
                    // whether Yarn or non-built-in versions of npm
                    "npm@built-in"
                ));
            let platform = Wrapper::new(width)
                .initial_indent(INDENTATION)
                .subsequent_indent(INDENTATION)
                .fill(&format!("platform:\n{}", platform_detail));
            format!("{}@{}\n{}\n{}", details.name, version, binaries, platform)
        }
        Package::Fetched(details) => {
            let package_info = format!("{}@{}", details.name, details.version);
            let footer_message = format!(
                "To make it available to execute, run `volta install {}`.",
                package_info
            );
            format!("{}\n\n{}", package_info, footer_message)
        }
    }
}

/// List a the source from a `Toolchain::Package`.
fn list_package_source(package: &Package) -> String {
    match package {
        Package::Default { .. } => String::from(" (default)"),
        Package::Project { path, .. } => format!(" (current @ {})", path.display()),
        Package::Fetched(..) => String::new(),
    }
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
            human::display_active, Node, PackageDetails, PackageManager, PackageManagerKind, Source,
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
        fn runtime_only_default() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (default)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Box::new(Node {
                source: Source::Default,
                version: NODE_12.clone(),
            }));
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

    Node: v12.2.0 (current @ ~/path/to/project.json)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Box::new(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            }));
            let package_manager = None;
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn runtime_and_yarn_default() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (default)
    Yarn: v1.16.0 (default)
    Tool binaries available: NONE

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Box::new(Node {
                source: Source::Default,
                version: NODE_12.clone(),
            }));
            let package_manager = Some(Box::new(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Default,
                version: YARN_VERSION.clone(),
            }));
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

            let runtime = Some(Box::new(Node {
                source: Source::Default,
                version: NODE_12.clone(),
            }));
            let package_manager = Some(Box::new(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            }));
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

            let runtime = Some(Box::new(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            }));
            let package_manager = Some(Box::new(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            }));
            let packages = vec![];

            assert_eq!(
                display_active(&runtime, &package_manager, &packages),
                expected
            );
        }

        #[test]
        fn with_default_tools() {
            let expected = "⚡️ Currently active tools:

    Node: v12.2.0 (current @ ~/path/to/project.json)
    Yarn: v1.16.0 (current @ ~/path/to/project.json)
    Tool binaries available:
        create-react-app (default)
        tsc, tsserver (default)

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Box::new(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            }));
            let package_manager = Some(Box::new(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            }));
            let packages = vec![
                Package::Default {
                    details: PackageDetails {
                        name: "create-react-app".to_string(),
                        version: Version::from((3, 0, 1)),
                    },
                    node: NODE_12.clone(),
                    tools: vec!["create-react-app".to_string()],
                },
                Package::Default {
                    details: PackageDetails {
                        name: "typescript".to_string(),
                        version: Version::from((3, 4, 3)),
                    },
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
        create-react-app (current @ ~/path/to/project.json)
        tsc, tsserver (default)

See options for more detailed reports by running `volta list --help`.";

            let runtime = Some(Box::new(Node {
                source: Source::Project(PROJECT_PATH.clone()),
                version: NODE_12.clone(),
            }));
            let package_manager = Some(Box::new(PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Source::Project(PROJECT_PATH.clone()),
                version: YARN_VERSION.clone(),
            }));
            let packages = vec![
                Package::Project {
                    details: PackageDetails {
                        name: "create-react-app".to_string(),
                        version: Version::from((3, 0, 1)),
                    },
                    path: PROJECT_PATH.clone(),
                    node: NODE_12.clone(),
                    tools: vec!["create-react-app".to_string()],
                },
                Package::Default {
                    details: PackageDetails {
                        name: "typescript".to_string(),
                        version: Version::from((3, 4, 3)),
                    },
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
                source: Source::Default,
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
                    source: Source::Default,
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
                source: Source::Default,
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

    v1.3.0
    v1.16.0 (default)
    v1.17.0 (current @ ~/path/to/project.json)";

            let yarns = [
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::None,
                    version: Version::from((1, 3, 0)),
                },
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Default,
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
        use crate::command::list::{Package, PackageDetails, Source};
        use semver::{Identifier, Version};

        #[test]
        fn none() {
            let expected = "⚡️ No tools or packages installed.

You can safely install packages by running `volta install <package name>`.
See `volta help install` for details and more options.";

            assert_eq!(display_packages(&[]), expected);
        }

        #[test]
        fn single_default() {
            let expected = "⚡️ Package versions in your toolchain:

    ember-cli@3.10.1 (default)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [Package::Default {
                details: PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                },
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ Package versions in your toolchain:

    ember-cli@3.10.1 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [Package::Project {
                details: PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                },
                path: PROJECT_PATH.clone(),
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn single_fetched() {
            let expected = "⚡️ Package versions in your toolchain:

    ember-cli@3.10.1
    
    To make it available to execute, run `volta install ember-cli@3.10.1`.";

            let packages = [Package::Fetched(PackageDetails {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
            })];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn multi_fetched() {
            let expected = "⚡️ Package versions in your toolchain:

    ember-cli@3.10.1
    
    To make it available to execute, run `volta install ember-cli@3.10.1`.
    ember-cli@3.8.2
    
    To make it available to execute, run `volta install ember-cli@3.8.2`.";

            let packages = [
                Package::Fetched(PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                }),
                Package::Fetched(PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                }),
            ];

            assert_eq!(display_packages(&packages), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ Package versions in your toolchain:

    ember-cli@3.10.1 (default)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in
    ember-cli@3.11.0--beta.3 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [
                Package::Default {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version::from((3, 10, 1)),
                    },
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package::Project {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version {
                            major: 3,
                            minor: 11,
                            patch: 0,
                            pre: vec![Identifier::AlphaNumeric("-beta.3".to_string())],
                            build: vec![],
                        },
                    },
                    node: NODE_12.clone(),
                    path: PROJECT_PATH.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_packages(&packages), expected);
        }
    }

    mod tools {
        use super::*;
        use crate::command::list::{Package, PackageDetails, Source};
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
            let expected = "⚡️ Tool `ember` available from:

    ember-cli@3.10.1 (default)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [Package::Default {
                details: PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                },
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn single_project() {
            let expected = "⚡️ Tool `ember` available from:

    ember-cli@3.10.1 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [Package::Project {
                details: PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                },
                path: PROJECT_PATH.clone(),
                node: NODE_12.clone(),
                tools: vec!["ember".to_string()],
            }];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn single_fetched() {
            let expected = "⚡️ Tool `ember` available from:

    ember-cli@3.10.1
    
    To make it available to execute, run `volta install ember-cli@3.10.1`.";

            let packages = [Package::Fetched(PackageDetails {
                name: "ember-cli".to_string(),
                version: Version::from((3, 10, 1)),
            })];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn multi_fetched() {
            let expected = "⚡️ Tool `ember` available from:

    ember-cli@3.10.1
    
    To make it available to execute, run `volta install ember-cli@3.10.1`.
    ember-cli@3.8.2
    
    To make it available to execute, run `volta install ember-cli@3.8.2`.";

            let packages = [
                Package::Fetched(PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 10, 1)),
                }),
                Package::Fetched(PackageDetails {
                    name: "ember-cli".to_string(),
                    version: Version::from((3, 8, 2)),
                }),
            ];

            assert_eq!(display_tool("ember", &packages), expected);
        }

        #[test]
        fn multi() {
            let expected = "⚡️ Tool `ember` available from:

    ember-cli@3.10.1 (default)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in
    ember-cli@3.11.0--beta.3 (current @ ~/path/to/project.json)
        binary tools: ember
        platform:
            runtime: node@12.2.0
            package manager: npm@built-in";

            let packages = [
                Package::Default {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version::from((3, 10, 1)),
                    },
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package::Project {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version {
                            major: 3,
                            minor: 11,
                            patch: 0,
                            pre: vec![Identifier::AlphaNumeric("-beta.3".to_string())],
                            build: vec![],
                        },
                    },
                    node: NODE_12.clone(),
                    path: PROJECT_PATH.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];

            assert_eq!(display_tool("ember", &packages), expected);
        }
    }

    mod all {
        use super::*;
        use crate::command::list::{PackageDetails, PackageManagerKind};
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
            v1.16.0 (default)
            v1.17.0 (current @ ~/path/to/project.json)
            v1.4.0

    Packages:
        typescript@3.4.3 (default)
            binary tools: tsc, tsserver
            platform:
                runtime: node@12.2.0
                package manager: npm@built-in
        typescript@3.5.1 (current @ ~/path/to/project.json)
            binary tools: tsc, tsserver
            platform:
                runtime: node@12.2.0
                package manager: npm@built-in
        ember-cli@3.10.1 (current @ ~/path/to/project.json)
            binary tools: ember
            platform:
                runtime: node@12.2.0
                package manager: npm@built-in
        ember-cli@3.8.2 (default)
            binary tools: ember
            platform:
                runtime: node@12.2.0
                package manager: npm@built-in";

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
                    source: Source::Default,
                    version: NODE_10.clone(),
                },
            ];

            let package_managers = [
                PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Default,
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
                Package::Default {
                    details: PackageDetails {
                        name: "typescript".to_string(),
                        version: Version::from((3, 4, 3)),
                    },
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package::Project {
                    details: PackageDetails {
                        name: "typescript".to_string(),
                        version: Version::from((3, 5, 1)),
                    },
                    path: PROJECT_PATH.clone(),
                    node: NODE_12.clone(),
                    tools: vec!["tsc".to_string(), "tsserver".to_string()],
                },
                Package::Project {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version::from((3, 10, 1)),
                    },
                    path: PROJECT_PATH.clone(),
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
                Package::Default {
                    details: PackageDetails {
                        name: "ember-cli".to_string(),
                        version: Version::from((3, 8, 2)),
                    },
                    node: NODE_12.clone(),
                    tools: vec!["ember".to_string()],
                },
            ];
            assert_eq!(
                display_all(&runtimes, &package_managers, &packages),
                expected
            );
        }
    }
}
