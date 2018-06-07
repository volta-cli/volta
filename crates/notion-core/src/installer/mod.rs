//! Provides types for installing tools into the Notion catalog.

pub mod node;
pub mod yarn;

use semver::Version;

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
