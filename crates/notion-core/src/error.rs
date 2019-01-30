use std::fmt;

use failure::Fail;
use notion_fail::{ExitCode, NotionFail};

use tool::ToolSpec;

#[derive(Debug, Fail)]
pub enum InternalError {
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
    RegistryFetchError {
        error: String,
    },
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InternalError::CreateDirError { dir, error } => {
                write!(f, "Could not create directory {}: {}", dir, error)
            }
            InternalError::DownloadToolNetworkError {
                tool,
                from_url,
                error,
            } => write!(
                f,
                "Failed to download {} from {}\n{}",
                tool, from_url, error
            ),
            InternalError::DownloadToolNotFound { tool } => write!(f, "{} not found", tool),
            InternalError::RegistryFetchError { error } => {
                write!(f, "Could not fetch public registry\n{}", error)
            }
        }
    }
}

impl NotionFail for InternalError {
    fn exit_code(&self) -> ExitCode {
        match self {
            InternalError::CreateDirError { .. } => ExitCode::FileSystemError,
            InternalError::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            InternalError::DownloadToolNotFound { .. } => ExitCode::NoVersionMatch,
            InternalError::RegistryFetchError { .. } => ExitCode::NetworkError,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
