//! Define the "plain" format style for list commands.

use std::fmt;

use semver::Version;

use volta_core::style::tool_version;

use super::{Package, Packager, Source, Toolchain};

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Source::Project(path) => format!(" (current @ {})", path.display()),
                Source::User => String::from(" (default)"),
                Source::None => String::from(""),
            }
        )
    }
}

pub(super) fn format(toolchain: &Toolchain) -> String {
    let runtimes = toolchain
        .node_runtimes
        .iter()
        .map(|runtime| display_node(&runtime.source, &runtime.version))
        .collect::<Vec<String>>()
        .join("\n");

    let packagers = toolchain
        .packagers
        .iter()
        .map(|packager| display_packager(&packager))
        .collect::<Vec<String>>()
        .join("\n");


    let packages = toolchain
        .packages
        .iter()
        .flat_map(display_package)
        .collect::<Vec<String>>()
        .join("\n");

    format!("{}\n{}\n{}", runtimes, packagers, packages)
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
                format(&Toolchain {
                    node_runtimes: vec![
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
