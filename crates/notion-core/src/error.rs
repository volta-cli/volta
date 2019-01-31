use std::fmt;

use failure::Fail;
use notion_fail::{ExitCode, NotionFail};
use semver::VersionReq;

use tool::ToolSpec;
use version::VersionSpec;

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
    NodeVersionNotFound {
        matching: VersionSpec,
    },
    PathError,
    RegistryFetchError {
        error: String,
    },
    YarnVersionNotFound {
        matching: VersionReq,
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
            ErrorDetails::NodeVersionNotFound { matching } => {
                write!(f, "No Node version found for {}", matching)
            }
            ErrorDetails::PathError => write!(f, "`path` internal error"),
            ErrorDetails::RegistryFetchError { error } => {
                write!(f, "Could not fetch public registry\n{}", error)
            }
            ErrorDetails::YarnVersionNotFound { matching } => {
                write!(f, "No Yarn version found for {}", matching)
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
            ErrorDetails::NodeVersionNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::PathError => ExitCode::UnknownError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::YarnVersionNotFound { .. } => ExitCode::NoVersionMatch,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
