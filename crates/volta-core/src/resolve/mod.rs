//! Provides methods for resolving a user-provided tool version requirement into a concrete version

use crate::error::ErrorDetails;

mod node;
mod package;
mod serial;
mod yarn;

pub use node::resolve as node;
pub use package::resolve as package;
pub use yarn::resolve as yarn;

fn registry_fetch_error(
    tool: impl AsRef<str>,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&reqwest::Error) -> ErrorDetails {
    let tool = tool.as_ref().to_string();
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::RegistryFetchError { tool, from_url }
}
