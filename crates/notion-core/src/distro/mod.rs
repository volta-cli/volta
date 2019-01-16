//! Provides types for fetching tool distributions into the local inventory.

mod error;
pub mod node;
pub mod yarn;

use hook::ToolHooks;
use inventory::Collection;
use notion_fail::Fallible;
use semver::Version;

/// The result of a requested installation.
pub enum Fetched<V> {
    /// Indicates that the given tool was already installed.
    Already(V),
    /// Indicates that the given tool was not already installed but has now been installed.
    Now(V),
}

impl<V> Fetched<V> {
    /// Consumes this value and produces the installed version.
    pub fn into_version(self) -> V {
        match self {
            Fetched::Already(version) | Fetched::Now(version) => version,
        }
    }

    /// Produces a reference to the installed version.
    pub fn version(&self) -> &V {
        match self {
            &Fetched::Already(ref version) | &Fetched::Now(ref version) => version,
        }
    }
}

pub trait Distro: Sized {
    type VersionDetails;

    /// Provisions a new Distro based on the Version and Possible Hooks
    fn new(version: Version, hooks: Option<&ToolHooks<Self>>) -> Fallible<Self>;

    /// Produces a reference to this distro's Tool version.
    fn version(&self) -> &Version;

    /// Fetches this version of the Tool. (It is left to the responsibility of the `Collection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &Collection<Self>) -> Fallible<Fetched<Self::VersionDetails>>;
}
