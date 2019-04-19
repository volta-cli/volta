//! Provides types for fetching tool distributions into the local inventory.

pub mod node;
pub mod package;
pub mod yarn;

use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::inventory::Collection;
use crate::tool::ToolSpec;
use notion_fail::Fallible;
use semver::Version;

/// The result of a requested installation.
#[derive(Debug)]
pub enum Fetched<V> {
    /// Indicates that the given tool was already fetched and unpacked.
    Already(V),
    /// Indicates that the given tool was not already fetched but has now been fetched and unpacked.
    Now(V),
    /// Indicates that the given tool is already installed.
    Installed(V),
}

impl<V> Fetched<V> {
    /// Consumes this value and produces the installed version.
    pub fn into_version(self) -> V {
        match self {
            Fetched::Already(version) | Fetched::Now(version) | Fetched::Installed(version) => {
                version
            }
        }
    }

    /// Produces a reference to the installed version.
    pub fn version(&self) -> &V {
        match self {
            &Fetched::Already(ref version)
            | &Fetched::Now(ref version)
            | &Fetched::Installed(ref version) => version,
        }
    }
}

pub trait Distro: Sized {
    type VersionDetails;
    type ResolvedVersion;

    /// Provisions a new Distro based on the name, Version and Possible Hooks
    fn new(
        name: String,
        version: Self::ResolvedVersion,
        hooks: Option<&ToolHooks<Self>>,
    ) -> Fallible<Self>;

    /// Produces a reference to this distro's Tool version.
    fn version(&self) -> &Version;

    /// Fetches this version of the Tool. (It is left to the responsibility of the `Collection`
    /// to update its state after fetching succeeds.)
    fn fetch(self, collection: &Collection<Self>) -> Fallible<Fetched<Self::VersionDetails>>;
}

fn download_tool_error(
    tool: ToolSpec,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&failure::Error) -> ErrorDetails {
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::DownloadToolNetworkError { tool, from_url }
}
