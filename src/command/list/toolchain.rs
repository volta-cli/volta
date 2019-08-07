use volta_core::inventory::Inventory;
use volta_fail::Fallible;

use super::{Filter, Node, Package, PackageManager};

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

impl Toolchain {
    pub(super) fn active(inventory: &Inventory, filter: &Filter) -> Fallible<Toolchain> {
        unimplemented!()
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
