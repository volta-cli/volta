use std::fmt;

use failure::Fail;
use notion_fail::{ExitCode, NotionFail};
use semver::VersionReq;

use tool::ToolSpec;
use version::VersionSpec;

#[derive(Debug, Fail)]
pub enum ErrorDetails {
    CannotPinPackage,
    CreateDirError {
        dir: String,
        error: String,
    },
    DepPackageReadError {
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
    NoHomeEnvironmentVar,
    NoLocalDataDir,
    NoPinnedNodeVersion,
    PackageReadError {
        error: String,
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
            ErrorDetails::CannotPinPackage => {
                write!(f, "Only node, yarn, and npm can be pinned in a project")
            }
            ErrorDetails::CreateDirError { dir, error } => {
                write!(f, "Could not create directory {}: {}", dir, error)
            }
            ErrorDetails::DepPackageReadError { error } => {
                write!(f, "Could not read dependent package info: {}", error)
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
            ErrorDetails::NoHomeEnvironmentVar => {
                write!(f, "environment variable 'HOME' is not set")
            }
            ErrorDetails::NoLocalDataDir => write!(f, "Windows LocalAppData directory not found"),
            ErrorDetails::NoPinnedNodeVersion => {
                write!(f, "There is no pinned node version for this project")
            }
            ErrorDetails::PackageReadError { error } => {
                write!(f, "Could not read package info: {}", error)
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
            ErrorDetails::CannotPinPackage => ExitCode::InvalidArguments,
            ErrorDetails::CreateDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DepPackageReadError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            ErrorDetails::DownloadToolNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::NodeVersionNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::NoHomeEnvironmentVar => ExitCode::EnvironmentError,
            ErrorDetails::NoLocalDataDir => ExitCode::EnvironmentError,
            ErrorDetails::NoPinnedNodeVersion => ExitCode::ConfigurationError,
            ErrorDetails::PackageReadError { .. } => ExitCode::FileSystemError,
            ErrorDetails::PathError => ExitCode::UnknownError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::YarnVersionNotFound { .. } => ExitCode::NoVersionMatch,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
