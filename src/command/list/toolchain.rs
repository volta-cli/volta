use std::path::PathBuf;
use std::rc::Rc;

use failure::ResultExt;
use semver::Version;

use super::{Filter, Node, Package, PackageManager, Source};
use crate::command::list::PackageManagerKind;
use volta_core::tool::PackageConfig;
use volta_core::{
    error::ErrorDetails, inventory::Inventory, platform::PlatformSpec, project::Project,
    session::Session,
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
                .and_then(|ref project_version| match project_version == version {
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
        filter: &Filter,
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
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        unimplemented!()
    }
}
