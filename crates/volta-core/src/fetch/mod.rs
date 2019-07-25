use crate::error::ErrorDetails;
use crate::tool;

mod node;
mod package;
mod yarn;

pub use yarn::fetch as yarn;

fn download_tool_error(
    tool: tool::Spec,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&failure::Error) -> ErrorDetails {
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::DownloadToolNetworkError { tool, from_url }
}
