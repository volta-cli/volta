//! Define the "plain" format style for list commands.

use semver::Version;

use volta_core::style::tool_version;

use super::{Node, Package, Packager, Source, Toolchain};

pub(super) fn format(toolchain: &Toolchain) -> Option<String> {
    let (runtimes, packagers, packages) = match toolchain {
        Toolchain::Node(runtimes) => (describe_runtimes(&runtimes), None, None),
        Toolchain::Packagers(packagers) => (None, describe_packagers(&packagers), None),
        Toolchain::Packages(packages) => (None, None, describe_packages(&packages)),
        Toolchain::Current {
            runtime,
            packager,
            packages,
        } => (
            runtime
                .as_ref()
                .and_then(|r| describe_runtimes(&[r.clone()])),
            packager
                .as_ref()
                .and_then(|p| describe_packagers(&[p.clone()])),
            describe_packages(&packages),
        ),
        Toolchain::All {
            runtimes,
            packagers,
            packages,
        } => (
            describe_runtimes(&runtimes),
            describe_packagers(&packagers),
            describe_packages(&packages),
        ),
    };

    match (runtimes, packagers, packages) {
        (Some(runtimes), Some(packagers), Some(packages)) => {
            Some(format!("{}\n{}\n{}", runtimes, packagers, packages))
        }
        (Some(runtimes), Some(packagers), None) => Some(format!("{}\n{}", runtimes, packagers)),
        (Some(runtimes), None, Some(packages)) => Some(format!("{}\n{}", runtimes, packages)),
        (Some(runtimes), None, None) => Some(format!("{}", runtimes)),
        (None, Some(packagers), Some(packages)) => Some(format!("{}\n{}", packagers, packages)),
        (None, Some(packagers), None) => Some(format!("{}", packagers)),
        (None, None, Some(packages)) => Some(format!("{}", packages)),
        (None, None, None) => None,
    }
}

fn describe_runtimes(runtimes: &[Node]) -> Option<String> {
    if runtimes.is_empty() {
        None
    } else {
        Some(
            runtimes
                .iter()
                .map(|runtime| display_node(&runtime.source, &runtime.version))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn describe_packagers(packagers: &[Packager]) -> Option<String> {
    if packagers.is_empty() {
        None
    } else {
        Some(
            packagers
                .iter()
                .map(|packager| display_packager(&packager))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn describe_packages(packages: &[Package]) -> Option<String> {
    if packages.is_empty() {
        None
    } else {
        Some(
            packages
                .iter()
                .flat_map(display_package)
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn display_node(source: &Source, version: &Version) -> String {
    format!("runtime {}{}", tool_version("node", version), source)
}

fn display_packager(packager: &Packager) -> String {
    format!(
        "packager {}{}",
        tool_version(&packager.type_, &packager.version),
        packager.source
    )
}

fn display_package(package: &Package) -> Vec<String> {
    package
        .tools
        .iter()
        .map(|tool_name| display_tool(&tool_name, package))
        .collect()
}

fn display_tool(name: &str, host: &Package) -> String {
    format!(
        "tool {} / {} {} {}{}",
        name,
        tool_version(&host.name, &host.version),
        tool_version("node", &host.node),
        // Should be updated when we support installing with custom packagers,
        // whether Yarn or non-built-in versions of npm
        "npm@built-in",
        host.source
    )
}

// These tests are organized by way of the *item* being printed, unlike in the
// `human` module, because the formatting is consistent across command formats.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use semver::Version;

    lazy_static! {
        static ref NODE_VERSION: Version = Version::from((12, 4, 0));
        static ref TYPESCRIPT_VERSION: Version = Version::from((3, 4, 1));
        static ref YARN_VERSION: Version = Version::from((1, 16, 0));
        static ref PROJECT_PATH: PathBuf = PathBuf::from("/a/b/c");
    }

    mod node {
        use super::super::*;
        use super::*;

        #[test]
        fn default() {
            let source = Source::User;
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0 (default)"
            );
        }

        #[test]
        fn project() {
            let source = Source::Project(PROJECT_PATH.clone());
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0 (current @ /a/b/c)"
            );
        }

        #[test]
        fn installed_not_set() {
            let source = Source::None;
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0"
            );
        }
    }

    mod yarn {
        use super::super::*;
        use super::*;
        use crate::command::list::*;

        #[test]
        fn default() {
            assert_eq!(
                display_packager(&Packager {
                    type_: PackagerType::Yarn,
                    source: Source::User,
                    version: YARN_VERSION.clone(),
                })
                .as_str(),
                "packager yarn@1.16.0 (default)"
            );
        }

        #[test]
        fn project() {
            assert_eq!(
                display_packager(&Packager {
                    type_: PackagerType::Yarn,
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: YARN_VERSION.clone()
                })
                .as_str(),
                "packager yarn@1.16.0 (current @ /a/b/c)"
            );
        }

        #[test]
        fn installed_not_set() {
            assert_eq!(
                display_packager(&Packager {
                    type_: PackagerType::Yarn,
                    source: Source::None,
                    version: YARN_VERSION.clone()
                })
                .as_str(),
                "packager yarn@1.16.0"
            );
        }
    }

    mod tool {
        use super::super::*;
        use super::*;

        #[test]
        fn default() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package {
                        name: "typescript".into(),
                        source: Source::User,
                        version: TYPESCRIPT_VERSION.clone(),
                        node: NODE_VERSION.clone(),
                        tools: vec![],
                    }
                )
                .as_str(),
                "tool tsc / typescript@3.4.1 node@12.4.0 npm@built-in (default)"
            );
        }

        #[test]
        fn project() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package {
                        name: "typescript".into(),
                        source: Source::Project(PROJECT_PATH.clone()),
                        version: TYPESCRIPT_VERSION.clone(),
                        node: NODE_VERSION.clone(),
                        tools: vec![],
                    }
                )
                .as_str(),
                "tool tsc / typescript@3.4.1 node@12.4.0 npm@built-in (current @ /a/b/c)"
            );
        }

        #[test]
        fn installed_not_set() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package {
                        name: "typescript".into(),
                        source: Source::None,
                        version: TYPESCRIPT_VERSION.clone(),
                        node: NODE_VERSION.clone(),
                        tools: vec!["tsc".into(), "tsserver".into()],
                    }
                )
                .as_str(),
                "tool tsc / typescript@3.4.1 node@12.4.0 npm@built-in"
            );
        }
    }

    mod toolchain {
        use super::super::*;
        use super::*;
        use crate::command::list::{Node, Packager, PackagerType, Toolchain};

        #[test]
        fn full() {
            assert_eq!(
                format(&Toolchain::All {
                    runtimes: vec![
                        Node {
                            source: Source::User,
                            version: NODE_VERSION.clone()
                        },
                        Node {
                            source: Source::None,
                            version: Version::from((8, 2, 4))
                        }
                    ],
                    packagers: vec![
                        Packager {
                            type_: PackagerType::Yarn,
                            source: Source::Project(PROJECT_PATH.clone()),
                            version: YARN_VERSION.clone()
                        },
                        Packager {
                            type_: PackagerType::Yarn,
                            source: Source::User,
                            version: Version::from((1, 17, 0))
                        }
                    ],
                    packages: vec![
                        Package {
                            name: "ember-cli".into(),
                            source: Source::User,
                            version: Version::from((3, 10, 2)),
                            node: NODE_VERSION.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package {
                            name: "ember-cli".into(),
                            source: Source::Project(PROJECT_PATH.clone()),
                            version: Version::from((3, 8, 1)),
                            node: NODE_VERSION.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package {
                            name: "typescript".into(),
                            source: Source::User,
                            version: TYPESCRIPT_VERSION.clone(),
                            node: NODE_VERSION.clone(),
                            tools: vec!["tsc".into(), "tsserver".into()]
                        }
                    ]
                })
                .expect("`format` with a non-empty toolchain returns `Some`")
                .as_str(),
                "runtime node@12.4.0 (default)\n\
                 runtime node@8.2.4\n\
                 packager yarn@1.16.0 (current @ /a/b/c)\n\
                 packager yarn@1.17.0 (default)\n\
                 tool ember / ember-cli@3.10.2 node@12.4.0 npm@built-in (default)\n\
                 tool ember / ember-cli@3.8.1 node@12.4.0 npm@built-in (current @ /a/b/c)\n\
                 tool tsc / typescript@3.4.1 node@12.4.0 npm@built-in (default)\n\
                 tool tsserver / typescript@3.4.1 node@12.4.0 npm@built-in (default)"
            )
        }
    }
}
