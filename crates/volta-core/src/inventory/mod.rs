//! Provides types for working with Volta's _inventory_, the local repository
//! of available tool versions.

mod node;
mod package;
mod yarn;

use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::path::Path;

use failure::ResultExt;
use lazycell::LazyCell;
use regex::Regex;
use semver::Version;
use volta_fail::Fallible;

use crate::{
    error::ErrorDetails,
    fs::read_dir_eager,
    tool::{Node, Package, PackageConfig, Tool, Yarn},
    version::VersionSpec,
};

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

/// The inventory of locally available tool versions.
pub struct Inventory {
    pub node: node::Collection,
    pub yarn: yarn::Collection,
    pub packages: package::Collection,
}

impl Inventory {
    /// Returns the current inventory.
    fn current() -> Fallible<Inventory> {
        Ok(Inventory {
            node: node::Collection::load()?,
            yarn: yarn::Collection::load()?,
            packages: package::Collection::load()?,
        })
    }
}

/// Reads the contents of a directory and returns the set of all versions found
/// in the directory's listing by matching filenames against the specified regex
/// and parsing the `version` named capture as a semantic version.
///
/// The regex should contain the `version` named capture by using the Rust regex
/// syntax `?P<version>`.
fn versions_matching(dir: &Path, re: &Regex) -> Fallible<BTreeSet<Version>> {
    let contents = read_dir_eager(dir).with_context(|_| ErrorDetails::ReadInventoryDirError {
        dir: dir.to_path_buf(),
    })?;

    let versions = contents
        .filter(|(_, metadata)| metadata.is_file())
        .filter_map(|(entry, _)| {
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            let captures = re.captures(&file_name)?;
            VersionSpec::parse_version(&captures["version"]).ok()
        })
        .collect();

    Ok(versions)
}
