//! Provides error types for the installer tools.

use archive::HttpError;
use notion_fail::{ExitCode, NotionFail};

use failure;
use reqwest::StatusCode;
use std::fmt;

// Once Issue #173 is implemented, we can use the ToolSpec struct to differentiate tools
#[derive(Debug)]
pub(crate) enum Tool {
    Node,
    Yarn,
}

impl fmt::Display for Tool {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match *self {
                Tool::Node => "Node",
                Tool::Yarn => "Yarn",
            }
        )
    }
}

#[derive(Debug, Fail, NotionFail)]
#[notion_fail(code = "NetworkError")]
pub(crate) enum DownloadError {
    NotFound { tool: Tool, version: String },
    Other { tool: Tool, version: String, from_url: String, error: String },
}

impl fmt::Display for DownloadError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::NotFound { tool, version } => {
                write!(fmt, "{} version {} not found", tool, version)
            }
            DownloadError::Other { tool, version, from_url, error } => {
                write!(fmt, "Failed to download {} version {} from {}\n{}", tool, version, from_url, error)
            }
        }
    }
}

impl DownloadError {
    pub(crate) fn for_tool_version(
        tool: Tool,
        version: String,
        from_url: String
    ) -> impl FnOnce(&failure::Error) -> DownloadError {
        move |error| match error.downcast_ref::<HttpError>() {
            Some(HttpError { code }) if *code == StatusCode::NotFound => DownloadError::NotFound {
                tool: tool,
                version: version,
            },
            _ => DownloadError::Other {
                tool: tool,
                version: version,
                from_url: from_url,
                error: error.to_string(),
            }
        }
    }
}
