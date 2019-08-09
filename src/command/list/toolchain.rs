use std::rc::Rc;

use semver::Version;

use super::{Filter, Node, Package, PackageManager, Source};
use crate::command::list::PackageManagerKind;
use volta_core::{
    inventory::Inventory, platform::PlatformSpec, project::Project, tool::PackageConfig,
};
use volta_fail::Fallible;

pub(super) enum Toolchain {
    Node(Vec<Node>),
    PackageManagers(Vec<PackageManager>),
    Packages(Vec<Package>),
    Tool {
        name: String,
        host_packages: Vec<Package>,
    },
    Active {
        runtime: Option<Node>,
        package_manager: Option<PackageManager>,
        packages: Vec<Package>,
    },
    All {
        runtimes: Vec<Node>,
        package_managers: Vec<PackageManager>,
        packages: Vec<Package>,
    },
}

/// Lightweight rule for which item to get the `Source` for.
enum Lookup {
    /// Look up the Node runtime
    Runtime,
    /// Look up the Yarn package manager
    Yarn,
}

impl Lookup {
    fn version_from_spec(self) -> impl Fn(Rc<PlatformSpec>) -> Option<Version> {
        move |spec| match self {
            Lookup::Runtime => Some(spec.node_runtime.clone()),
            Lookup::Yarn => spec.yarn.clone(),
        }
    }

    fn version_source<'p>(
        self,
        project: &'p Option<Rc<Project>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        version: &Version,
    ) -> Source {
        match project {
            Some(project) => project
                .platform()
                .and_then(self.version_from_spec())
                .and_then(|project_version| match &project_version == version {
                    true => Some(Source::Project(project.package_file())),
                    false => None,
                }),
            None => user_platform
                .clone()
                .and_then(self.version_from_spec())
                .and_then(|ref default_version| match default_version == version {
                    true => Some(Source::Default),
                    false => None,
                }),
        }
        .unwrap_or(Source::None)
    }

    /// Determine the `Source` for a given kind of tool (`Lookup`).
    fn active_tool(
        self,
        project: &Option<Rc<Project>>,
        user: &Option<Rc<PlatformSpec>>,
    ) -> Option<(Source, Version)> {
        match project {
            Some(project) => project
                .platform()
                .and_then(self.version_from_spec())
                .map(|version| (Source::Project(project.package_file()), version)),
            None => user
                .clone()
                .and_then(self.version_from_spec())
                .map(|version| (Source::Default, version)),
        }
    }
}

fn package_source(name: &str, version: &Version, project: &Option<Rc<Project>>) -> Source {
    match project {
        Some(project) if project.has_dependency(name, version) => {
            Source::Project(project.package_file())
        }
        _ => Source::Default,
    }
}

impl Toolchain {
    pub(super) fn active(
        project: &Option<Rc<Project>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        inventory: &Inventory,
    ) -> Fallible<Toolchain> {
        let runtime = Lookup::Runtime
            .active_tool(project, user_platform)
            .map(|(source, version)| Node { source, version });

        let package_manager =
            Lookup::Yarn
                .active_tool(project, user_platform)
                .map(|(source, version)| PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source,
                    version,
                });

        let packages = inventory
            .packages
            .clone()
            .into_iter()
            .map(|config| Package {
                // Putting this first lets us borrow here, then move everything
                // into the `Package` after.
                source: package_source(&config.name, &config.version, &project),
                name: config.name,
                version: config.version,
                node: config.platform.node_runtime,
                tools: config.bins,
            })
            .collect();

        Ok(Toolchain::Active {
            runtime,
            package_manager,
            packages,
        })
    }

    pub(super) fn all(
        project: &Option<Rc<Project>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        inventory: &Inventory,
    ) -> Fallible<Toolchain> {
        let runtimes = inventory
            .node
            .versions
            .iter()
            .map(|version| Node {
                source: Lookup::Runtime.version_source(project, user_platform, version),
                version: version.clone(),
            })
            .collect();

        let package_managers = inventory
            .yarn
            .versions
            .iter()
            .map(|version| PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Lookup::Yarn.version_source(project, user_platform, version),
                version: version.clone(),
            })
            .collect();

        let packages = inventory
            .packages
            .clone()
            .into_iter()
            .map(|config| Package {
                source: package_source(&config.name, &config.version, &project),
                name: config.name,
                version: config.version,
                node: config.platform.node_runtime,
                tools: config.bins,
            })
            .collect();

        Ok(Toolchain::All {
            runtimes,
            package_managers,
            packages,
        })
    }

    pub(super) fn node(
        inventory: &Inventory,
        project: &Option<Rc<Project>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        filter: &Filter,
    ) -> Toolchain {
        let runtimes = inventory
            .node
            .versions
            .iter()
            .filter_map(|version| {
                let source = Lookup::Runtime.version_source(project, user_platform, version);
                if source.allowed_with(filter) {
                    let version = version.clone();
                    Some(Node { source, version })
                } else {
                    None
                }
            })
            .collect();

        Toolchain::Node(runtimes)
    }

    pub(super) fn yarn(
        inventory: &Inventory,
        project: &Option<Rc<Project>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        filter: &Filter,
    ) -> Toolchain {
        let yarns = inventory
            .yarn
            .versions
            .iter()
            .filter_map(|version| {
                let source = Lookup::Yarn.version_source(project, user_platform, version);
                if source.allowed_with(filter) {
                    Some(PackageManager {
                        kind: PackageManagerKind::Yarn,
                        source,
                        version: version.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Toolchain::PackageManagers(yarns)
    }

    pub(super) fn package_or_tool(
        name: &str,
        inventory: &Inventory,
        project: &Option<Rc<Project>>,
        filter: &Filter,
    ) -> Toolchain {
        /// An internal-only helper for tracking whether we found a given item
        /// from the `PackageCollection` as a *package* or as a *tool*.
        #[derive(PartialEq)]
        enum Kind {
            Package,
            Tool,
        }

        let packages_and_tools = inventory
            .packages
            .clone()
            .into_iter()
            .filter_map(|config| {
                let source = package_source(&config.name, &config.version, project);
                if source.allowed_with(filter) {
                    // Start with the package itself, since tools often match
                    // the package name and we prioritize packages.
                    if &config.name == name {
                        Some((Kind::Package, config, source))
                    } else if config
                        .bins
                        .iter()
                        .find(|bin| bin.as_str() == name)
                        .is_some()
                    {
                        Some((Kind::Tool, config, source))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<(Kind, PackageConfig, Source)>>();

        let has_packages = packages_and_tools
            .iter()
            .any(|(kind, ..)| kind == &Kind::Package);

        let has_tools = packages_and_tools
            .iter()
            .any(|(kind, ..)| kind == &Kind::Tool);

        match (has_packages, has_tools) {
            // If there are neither packages nor tools, treat it as `Packages`,
            // but don't re-process the data just to construct an empty `Vec`!
            (false, false) => Toolchain::Packages(vec![]),
            // If there are any packages, we resolve this *as* `Packages`, even
            // if there are also matching tools, since we give priority to
            // listing packages between packages and tools.
            (true, _) => {
                let packages = packages_and_tools
                    .into_iter()
                    .filter_map(|(kind, config, source)| match kind {
                        Kind::Package => Some(Package {
                            name: config.name,
                            source,
                            version: config.version,
                            node: config.platform.node_runtime,
                            tools: config.bins,
                        }),
                        Kind::Tool => None,
                    })
                    .collect();

                Toolchain::Packages(packages)
            }
            // If there are no packages matching, but we do have tools matching,
            // we return `Tool`.
            (false, true) => {
                let host_packages = packages_and_tools
                    .into_iter()
                    .filter_map(|(kind, config, source)| match kind {
                        Kind::Tool => Some(Package {
                            name: config.name,
                            source,
                            version: config.version,
                            node: config.platform.node_runtime,
                            tools: config.bins,
                        }),
                        Kind::Package => None, // should be none of these!
                    })
                    .collect();

                Toolchain::Tool {
                    name: name.into(),
                    host_packages,
                }
            }
        }
    }
}
