//! Provides types for fetching tool distributions into the local inventory.

pub mod node;
pub mod yarn;

use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::inventory::Collection;
use crate::tool::ToolSpec;
use archive::HttpError;
use notion_fail::Fallible;
use reqwest::StatusCode;
use semver::Version;

/// The result of a requested installation.
#[derive(Debug)]
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

pub fn download_tool_error(
    toolspec: ToolSpec,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&failure::Error) -> ErrorDetails {
    let from_url = from_url.as_ref().to_string();
    move |error| match error.downcast_ref::<HttpError>() {
        Some(HttpError {
            code: StatusCode::NOT_FOUND,
        }) => ErrorDetails::DownloadToolNotFound { tool: toolspec },
        Some(_) | None => ErrorDetails::DownloadToolNetworkError {
            tool: toolspec,
            error: error.to_string(),
            from_url,
        },
    }
}
