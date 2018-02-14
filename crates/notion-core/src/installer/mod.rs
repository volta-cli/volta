//! Provides types and traits for installing tools into the Notion catalog.

pub mod node;

use semver::Version;

pub enum Installed {
    Already(Version),
    Now(Version)
}

impl Installed {
    pub fn into_version(self) -> Version {
        match self {
              Installed::Already(version)
            | Installed::Now(version) => version
        }
    }

    pub fn version(&self) -> &Version {
        match self {
              &Installed::Already(ref version)
            | &Installed::Now(ref version) => version
        }
    }
}
