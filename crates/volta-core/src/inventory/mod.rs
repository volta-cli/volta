//! Provides types for working with Volta's _inventory_, the local repository
//! of available tool versions.

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use crate::tool::{Node, Package, PackageConfig, Tool, Yarn};
use lazycell::LazyCell;
use semver::Version;
use volta_fail::Fallible;

pub(crate) mod serial;

/// Lazily loaded inventory.
pub struct LazyInventory {
    inventory: LazyCell<Inventory>,
}

impl LazyInventory {
    /// Constructs a new `LazyInventory`.
    pub fn new() -> LazyInventory {
        LazyInventory {
            inventory: LazyCell::new(),
        }
    }

    /// Forces the loading of the inventory and returns an immutable reference to it.
    pub fn get(&self) -> Fallible<&Inventory> {
        self.inventory.try_borrow_with(|| Inventory::current())
    }

    /// Forces the loading of the inventory and returns a mutable reference to it.
    pub fn get_mut(&mut self) -> Fallible<&mut Inventory> {
        self.inventory.try_borrow_mut_with(|| Inventory::current())
    }
}

/// The common operations to perform on a collection managed by Volta.
pub trait Collection {
    /// The kind of tool represented by the collection.
    type Tool: Tool;

    /// Add a new version to the collection.
    fn add(&mut self, version: &Version) -> Fallible<()>;

    /// Remove a version from the collection.
    fn remove(&mut self, version: &Version) -> Fallible<()>;
}

pub struct NodeCollection {
    pub versions: BTreeSet<Version>,
}

pub struct YarnCollection {
    pub versions: BTreeSet<Version>,
}

pub struct PackageCollection {
    pub packages: BTreeSet<PackageConfig>,
}

impl Collection for NodeCollection {
    type Tool = Node;

    fn add(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }

    fn remove(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }
}

impl Collection for YarnCollection {
    type Tool = Yarn;

    fn add(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }

    fn remove(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }
}

impl Collection for PackageCollection {
    type Tool = Package;

    fn add(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }

    fn remove(&mut self, version: &Version) -> Fallible<()> {
        unimplemented!()
    }
}

/// The inventory of locally available tool versions.
pub struct Inventory {
    pub node: NodeCollection,
    pub yarn: YarnCollection,
    pub packages: PackageCollection,
}

impl Inventory {
    /// Returns the current inventory.
    fn current() -> Fallible<Inventory> {
        Ok(Inventory {
            node: NodeCollection::load()?,
            yarn: YarnCollection::load()?,
            packages: PackageCollection::load()?,
        })
    }
}
