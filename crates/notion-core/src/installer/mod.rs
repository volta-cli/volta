//! Provides types for installing tools into the Notion catalog.

pub mod node;
pub mod yarn;

use std::fs::{File};
use semver::Version;
use notion_fail::{Fallible};
use catalog::Collection;

/// The result of a requested installation.
pub enum Installed {
    /// Indicates that the given tool was already installed.
    Already(Version),
    /// Indicates that the given tool was not already installed but has now been installed.
    Now(Version),
}

impl Installed {
    /// Consumes this value and produces the installed version.
    pub fn into_version(self) -> Version {
        match self {
            Installed::Already(version) | Installed::Now(version) => version,
        }
    }

    /// Produces a reference to the installed version.
    pub fn version(&self) -> &Version {
        match self {
            &Installed::Already(ref version) | &Installed::Now(ref version) => version,
        }
    }
}

pub trait Install:Sized {
    /// Provision an `Installer` from the public distributor (e.g. `https://nodejs.org`).
    fn public(version: Version) -> Fallible<Self>;

    /// Provision an `Installer` from a remote distributor.
    fn remote(version: Version, url: &str) -> Fallible<Self>;

    /// Provision an `Installer` from the filesystem.
    fn cached(version: Version, file: File) -> Fallible<Self>;

    /// Produces a reference to this installer's Tool version.
    fn version(&self) -> &Version;

    /// Installs this version of the Tool. (It is left to the responsibility of the `Collection`
    /// to update its state after installation succeeds.)
    fn install(self, catalog: &Collection<Self>) -> Fallible<Installed>;
}
