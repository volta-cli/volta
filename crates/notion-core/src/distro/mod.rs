//! Provides types for fetching tool distributions into the local inventory.

mod error;
pub mod node;
pub mod yarn;

use crate::inventory::Collection;
use notion_fail::Fallible;
use semver::Version;
use std::fmt::{self, Display, Formatter};
use std::fs::File;

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

/// Abstraction to contain info about Distro versions.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum DistroVersion {
    // the version of the Node runtime, and the npm version installed with that
    Node(Version, Version),
    Yarn(Version),
    Npm(Version),
    Package(String, Version),
}

impl Display for DistroVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &DistroVersion::Node(ref runtime, ref npm) => {
                format!("node version {} (npm {})", runtime, npm)
            }
            &DistroVersion::Yarn(ref version) => format!("yarn version {}", version),
            &DistroVersion::Npm(ref version) => format!("npm version {}", version),
            &DistroVersion::Package(ref name, ref version) => {
                format!("{} version {}", name, version)
            }
        };
        f.write_str(&s)
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
    fn fetch(self, collection: &Collection<Self>) -> Fallible<Fetched<DistroVersion>>;
}
