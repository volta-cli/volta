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

fn lookup_version(lookup: Lookup, spec: &Rc<PlatformSpec>) -> Option<Version> {
    match lookup {
        Lookup::Runtime => Some(spec.node_runtime.clone()),
        Lookup::Yarn => spec.yarn.clone(),
    }
}

/// Determine the `Source` for a given kind of tool (`Lookup`).
fn source(
    project: &Option<Rc<PlatformSpec>>,
    user: &Option<Rc<PlatformSpec>>,
    cwd: &PathBuf,
    lookup: Lookup,
) -> Option<(Source, Version)> {
    if let Some(project) = project {
        lookup_version(lookup, project).map(|version| (Source::Project(cwd.clone()), version))
    } else if let Some(user) = user {
        lookup_version(lookup, user).map(|version| (Source::User, version))
    } else {
        None
    }
}

fn package_source(name: &str, version: &Version, project: &Option<Rc<Project>>) -> Source {
    match project {
        Some(project) if project.has_dependency(name, version) => {
            Source::Project(project.package_file())
        }
        _ => Source::User,
    }
}

impl Toolchain {
    pub(super) fn active(
        project_platform: &Option<Rc<PlatformSpec>>,
        user_platform: &Option<Rc<PlatformSpec>>,
        project: &Option<Rc<Project>>,
        inventory: &Inventory,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        let cwd = std::env::current_dir().with_context(|_| ErrorDetails::CurrentDirError)?;

        let runtime = source(&project_platform, &user_platform, &cwd, Lookup::Runtime)
            .map(|(source, version)| Node { source, version });

        let package_manager = source(&project_platform, &user_platform, &cwd, Lookup::Yarn).map(
            |(source, version)| PackageManager {
                kind: PackageManagerKind::Yarn,
                source,
                version,
            },
        );

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

    pub(super) fn all(inventory: &Inventory) -> Fallible<Toolchain> {
        unimplemented!()
    }

    pub(super) fn node(inventory: &Inventory, filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }

    pub(super) fn yarn(inventory: &Inventory, filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
    }

    pub(super) fn package_or_tool(
        name: &str,
        inventory: &Inventory,
        filter: &Filter,
    ) -> Fallible<Toolchain> {
        unimplemented!()
    }
}
