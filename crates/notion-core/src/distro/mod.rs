//! Provides types for fetching tool distributions into the local inventory.

mod error;
pub mod node;
pub mod yarn;

use inventory::Collection;
use notion_fail::Fallible;
use semver::Version;
use std::fs::File;

/// The result of a requested installation.
pub enum Fetched {
    /// Indicates that the given tool was already installed.
    Already(Version),
    /// Indicates that the given tool was not already installed but has now been installed.
    Now(Version),
}

impl Fetched {
    /// Consumes this value and produces the installed version.
    pub fn into_version(self) -> Version {
        match self {
            Fetched::Already(version) | Fetched::Now(version) => version,
        }
    }

    /// Produces a reference to the installed version.
    pub fn version(&self) -> &Version {
        match self {
            &Fetched::Already(ref version) | &Fetched::Now(ref version) => version,
        }
    }
}

pub trait Distro: Sized {
    /// Provision a distribution from the public distributor (e.g. `https://nodejs.org`).
    fn public(version: Version) -> Fallible<Self>;

    /// Provision a distribution from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self>;

    /// Provision a distribution from the filesystem.
    fn local(version: Version, file: File) -> Fallible<Self>;

    /// Produces a reference to this distro's Tool version.
    fn version(&self) -> &Version;

    /// Fetches this version of the Tool. (It is left to the responsibility of the `Collection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, catalog: &Collection<Self>) -> Fallible<Fetched>;
}
