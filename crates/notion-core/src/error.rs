use std::fmt;

use failure::Fail;
use notion_fail::{ExitCode, NotionFail};

use tool::ToolSpec;

#[derive(Debug, Fail)]
pub enum ErrorDetails {
    CreateDirError {
        dir: String,
        error: String,
    },
    DownloadToolNetworkError {
        tool: ToolSpec,
        from_url: String,
        error: String,
    },
    DownloadToolNotFound {
        tool: ToolSpec,
    },
    PathError,
    RegistryFetchError {
        error: String,
    },
}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorDetails::CreateDirError { dir, error } => {
                write!(f, "Could not create directory {}: {}", dir, error)
            }
            ErrorDetails::DownloadToolNetworkError {
                tool,
                from_url,
                error,
            } => write!(
                f,
                "Failed to download {} from {}\n{}",
                tool, from_url, error
            ),
            ErrorDetails::DownloadToolNotFound { tool } => write!(f, "{} not found", tool),
            ErrorDetails::PathError => write!(f, "`path` internal error"),
            ErrorDetails::RegistryFetchError { error } => {
                write!(f, "Could not fetch public registry\n{}", error)
            }
        }
    }
}

impl NotionFail for ErrorDetails {
    fn exit_code(&self) -> ExitCode {
        match self {
            ErrorDetails::CreateDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            ErrorDetails::DownloadToolNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::PathError => ExitCode::UnknownError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
