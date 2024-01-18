use super::{Filter, Node, Package, PackageManager, Source};
use crate::command::list::PackageManagerKind;
use node_semver::Version;
use volta_core::error::Fallible;
use volta_core::inventory::{
    node_versions, npm_versions, package_configs, pnpm_versions, yarn_versions,
};
use volta_core::platform::PlatformSpec;
use volta_core::project::Project;
use volta_core::tool::PackageConfig;

pub(super) enum Toolchain {
    Node(Vec<Node>),
    PackageManagers {
        kind: PackageManagerKind,
        managers: Vec<PackageManager>,
    },
    Packages(Vec<Package>),
    Tool {
        name: String,
        host_packages: Vec<Package>,
    },
    Active {
        runtime: Option<Box<Node>>,
        package_managers: Vec<PackageManager>,
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
    /// Look up the npm package manager
    Npm,
    /// Look up the pnpm package manager
    Pnpm,
    /// Look up the Yarn package manager
    Yarn,
}

impl Lookup {
    fn version_from_spec(&self) -> impl Fn(&PlatformSpec) -> Option<Version> + '_ {
        move |spec| match self {
            Lookup::Runtime => Some(spec.node.clone()),
            Lookup::Npm => spec.npm.clone(),
            Lookup::Pnpm => spec.pnpm.clone(),
            Lookup::Yarn => spec.yarn.clone(),
        }
    }

    fn version_source(
        self,
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
        version: &Version,
    ) -> Source {
        project
            .and_then(|proj| {
                proj.platform()
                    .and_then(self.version_from_spec())
                    .and_then(|project_version| {
                        if &project_version == version {
                            Some(Source::Project(proj.manifest_file().to_owned()))
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                default_platform
                    .and_then(self.version_from_spec())
                    .and_then(|default_version| {
                        if &default_version == version {
                            Some(Source::Default)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(Source::None)
    }

    /// Determine the `Source` for a given kind of tool (`Lookup`).
    fn active_tool(
        self,
        project: Option<&Project>,
        default: Option<&PlatformSpec>,
    ) -> Option<(Source, Version)> {
        project
            .and_then(|proj| {
                proj.platform()
                    .and_then(self.version_from_spec())
                    .map(|version| (Source::Project(proj.manifest_file().to_owned()), version))
            })
            .or_else(|| {
                default
                    .and_then(self.version_from_spec())
                    .map(|version| (Source::Default, version))
            })
    }
}

/// Look up the `Source` for a tool with a given name.
fn tool_source(name: &str, project: Option<&Project>) -> Fallible<Source> {
    match project {
        Some(project) => {
            if project.has_direct_bin(name.as_ref())? {
                Ok(Source::Project(project.manifest_file().to_owned()))
            } else {
                Ok(Source::Default)
            }
        }
        _ => Ok(Source::Default),
    }
}

impl Toolchain {
    pub(super) fn active(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
    ) -> Fallible<Toolchain> {
        let runtime = Lookup::Runtime
            .active_tool(project, default_platform)
            .map(|(source, version)| Box::new(Node { source, version }));

        let package_managers =
            Lookup::Npm
                .active_tool(project, default_platform)
                .map(|(source, version)| PackageManager {
                    kind: PackageManagerKind::Npm,
                    source,
                    version,
                })
                .into_iter()
                .chain(Lookup::Pnpm.active_tool(project, default_platform).map(
                    |(source, version)| PackageManager {
                        kind: PackageManagerKind::Pnpm,
                        source,
                        version,
                    },
                ))
                .chain(Lookup::Yarn.active_tool(project, default_platform).map(
                    |(source, version)| PackageManager {
                        kind: PackageManagerKind::Yarn,
                        source,
                        version,
                    },
                ))
                .collect();

        let packages = Package::from_inventory_and_project(project)?;

        Ok(Toolchain::Active {
            runtime,
            package_managers,
            packages,
        })
    }

    pub(super) fn all(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
    ) -> Fallible<Toolchain> {
        let runtimes = node_versions()?
            .iter()
            .map(|version| Node {
                source: Lookup::Runtime.version_source(project, default_platform, version),
                version: version.clone(),
            })
            .collect();

        let package_managers = npm_versions()?
            .iter()
            .map(|version| PackageManager {
                kind: PackageManagerKind::Npm,
                source: Lookup::Npm.version_source(project, default_platform, version),
                version: version.clone(),
            })
            .chain(pnpm_versions()?.iter().map(|version| PackageManager {
                kind: PackageManagerKind::Pnpm,
                source: Lookup::Pnpm.version_source(project, default_platform, version),
                version: version.clone(),
            }))
            .chain(yarn_versions()?.iter().map(|version| PackageManager {
                kind: PackageManagerKind::Yarn,
                source: Lookup::Yarn.version_source(project, default_platform, version),
                version: version.clone(),
            }))
            .collect();

        let packages = Package::from_inventory_and_project(project)?;

        Ok(Toolchain::All {
            runtimes,
            package_managers,
            packages,
        })
    }

    pub(super) fn node(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        let runtimes = node_versions()?
            .iter()
            .filter_map(|version| {
                let source = Lookup::Runtime.version_source(project, default_platform, version);
                if source.allowed_with(filter) {
                    let version = version.clone();
                    Some(Node { source, version })
                } else {
                    None
                }
            })
            .collect();

        Ok(Toolchain::Node(runtimes))
    }

    pub(super) fn npm(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        let managers = npm_versions()?
            .iter()
            .filter_map(|version| {
                let source = Lookup::Npm.version_source(project, default_platform, version);
                if source.allowed_with(filter) {
                    Some(PackageManager {
                        kind: PackageManagerKind::Npm,
                        source,
                        version: version.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Toolchain::PackageManagers {
            kind: PackageManagerKind::Npm,
            managers,
        })
    }

    pub(super) fn pnpm(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        let managers = pnpm_versions()?
            .iter()
            .filter_map(|version| {
                let source = Lookup::Pnpm.version_source(project, default_platform, version);
                if source.allowed_with(filter) {
                    Some(PackageManager {
                        kind: PackageManagerKind::Pnpm,
                        source,
                        version: version.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Toolchain::PackageManagers {
            kind: PackageManagerKind::Pnpm,
            managers,
        })
    }

    pub(super) fn yarn(
        project: Option<&Project>,
        default_platform: Option<&PlatformSpec>,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        let managers = yarn_versions()?
            .iter()
            .filter_map(|version| {
                let source = Lookup::Yarn.version_source(project, default_platform, version);
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

        Ok(Toolchain::PackageManagers {
            kind: PackageManagerKind::Yarn,
            managers,
        })
    }

    pub(super) fn package_or_tool(
        name: &str,
        project: Option<&Project>,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        /// An internal-only helper for tracking whether we found a given item
        /// from the `PackageCollection` as a *package* or as a *tool*.
        #[derive(PartialEq, Debug)]
        enum Kind {
            Package,
            Tool,
        }

        /// A convenient name for this tuple, since we have to name it in a few
        /// spots below.
        type Triple<'p> = (Kind, &'p PackageConfig, Source);

        let configs = package_configs()?;
        let packages_and_tools = configs
            .iter()
            .filter_map(|config| {
                // Start with the package itself, since tools often match
                // the package name and we prioritize packages.
                if config.name == name {
                    let source = Package::source(name, project);
                    if source.allowed_with(filter) {
                        Some(Ok((Kind::Package, config, source)))
                    } else {
                        None
                    }

                // Then check if the passed name matches an installed package's
                // binaries. If it does, we have a tool.
                } else if config.bins.iter().any(|bin| bin.as_str() == name) {
                    tool_source(name, project)
                        .map(|source| {
                            if source.allowed_with(filter) {
                                Some((Kind::Tool, config, source))
                            } else {
                                None
                            }
                        })
                        .transpose()

                // Otherwise, we don't have any match all.
                } else {
                    None
                }
            })
            // Then eagerly collect the first error (if there are any) and
            // return it; otherwise we have a totally valid collection.
            .collect::<Fallible<Vec<Triple>>>()?;

        let (has_packages, has_tools) =
            packages_and_tools
                .iter()
                .fold((false, false), |(packages, tools), (kind, ..)| {
                    (
                        packages || kind == &Kind::Package,
                        tools || kind == &Kind::Tool,
                    )
                });

        let toolchain = match (has_packages, has_tools) {
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
                        Kind::Package => Some(Package::new(config, &source)),
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
                        Kind::Tool => Some(Package::new(config, &source)),
                        Kind::Package => None, // should be none of these!
                    })
                    .collect();

                Toolchain::Tool {
                    name: name.into(),
                    host_packages,
                }
            }
        };

        Ok(toolchain)
    }
}
