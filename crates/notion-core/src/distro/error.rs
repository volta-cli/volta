//! Provides error types for the installer tools.

use crate::tool::ToolSpec;
use archive::HttpError;
use notion_fail::{ExitCode, NotionFail};

use failure::Fail;
use reqwest::StatusCode;
use std::fmt;

#[derive(Debug, Fail)]
pub(crate) enum DownloadError {
    NotFound {
        toolspec: ToolSpec,
    },
    Other {
        toolspec: ToolSpec,
        from_url: String,
        error: String,
    },
}

impl NotionFail for DownloadError {
    fn is_user_friendly(&self) -> bool {
        true
    }

    fn exit_code(&self) -> ExitCode {
        match self {
            DownloadError::NotFound { .. } => ExitCode::NoVersionMatch,
            DownloadError::Other { .. } => ExitCode::NetworkError,
        }
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::NotFound { toolspec } => write!(fmt, "{} not found", toolspec),
            DownloadError::Other {
                toolspec,
                from_url,
                error,
            } => write!(
                fmt,
                "Failed to download {} from {}\n{}",
                toolspec, from_url, error
            ),
        }
    }
}

impl DownloadError {
    pub(crate) fn for_tool(
        toolspec: ToolSpec,
        from_url: String,
    ) -> impl FnOnce(&failure::Error) -> DownloadError {
        move |error| {
            if let Some(HttpError {
                code: StatusCode::NOT_FOUND,
            }) = error.downcast_ref::<HttpError>()
            {
                DownloadError::NotFound { toolspec: toolspec }
            } else {
                DownloadError::Other {
                    toolspec: toolspec,
                    from_url: from_url,
                    error: error.to_string(),
                }
            }
        }
    }
}
