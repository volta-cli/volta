use std::fmt;

use failure::Fail;
use notion_fail::{ExitCode, NotionFail};

use crate::tool::ToolSpec;

#[derive(Debug, Fail)]
pub enum ErrorDetails {
    /// Thrown when package tries to install a binary that is already installed.
    BinaryAlreadyInstalled {
        bin_name: String,
        existing_package: String,
        new_package: String,
    },

    BinaryExecError,

    /// Thrown when a binary could not be found in the local inventory
    BinaryNotFound {
        name: String,
    },

    /// Thrown when a user tries to `notion pin` something other than node/yarn/npm.
    CannotPinPackage {
        package: String,
    },

    CommandNotImplemented {
        command_name: String,
    },

    CouldNotDetermineTool,

    CreateDirError {
        dir: String,
    },

    /// Thrown when deleting a directory fails
    DeleteDirectoryError {
        directory: String,
    },

    /// Thrown when deleting a file fails
    DeleteFileError {
        file: String,
    },

    /// Thrown when reading dependency package info fails
    DepPackageReadError,

    DeprecatedCommandError {
        command: String,
        advice: String,
    },

    DownloadToolNetworkError {
        tool: ToolSpec,
        from_url: String,
    },

    InvalidHookCommand {
        command: String,
    },

    /// Thrown when BinConfig (read from file) does not contain Platform info.
    NoBinPlatform {
        binary: String,
    },

    /// Thrown when there is no Node version matching a requested semver specifier.
    NodeVersionNotFound {
        matching: String,
    },

    NoGlobalInstalls,

    NoHomeEnvironmentVar,

    NoLocalDataDir,

    /// Thrown when a user tries to install or fetch a package with no executables.
    NoPackageExecutables,

    /// Thrown when a user tries to pin a Yarn version before pinning a Node version.
    NoPinnedNodeVersion,

    /// Thrown when the platform (Node version) could not be determined
    NoPlatform,

    /// Thrown when Yarn is not set in a project
    NoProjectYarn,

    /// Thrown when the user tries to pin Node or Yarn versions outside of a package.
    NotInPackage,

    /// Thrown when default Yarn is not set
    NoUserYarn,

    NoVersionsFound,

    NpxNotAvailable {
        version: String,
    },

    /// Thrown when package install command is not successful.
    PackageInstallFailed,

    /// Thrown when there is an error fetching package metadata
    PackageMetadataFetchError {
        from_url: String,
    },

    /// Thrown when parsing a package manifest fails
    PackageParseError {
        file: String,
    },

    /// Thrown when reading a package manifest fails
    PackageReadError {
        file: String,
    },

    /// Thrown when a package has been unpacked but is not formed correctly.
    PackageUnpackError,

    /// Thrown when there is no package version matching a requested semver specifier.
    PackageVersionNotFound {
        name: String,
        matching: String,
    },

    PathError,

    /// Thrown when the public registry for Node or Yarn could not be downloaded.
    RegistryFetchError {
        tool: String,
        from_url: String,
    },

    /// Thrown when Notion is unable to create a shim
    ShimCreateError {
        name: String,
    },

    /// Thrown when trying to remove a built-in shim (`node`, `yarn`, etc.)
    ShimRemoveBuiltInError {
        name: String,
    },

    /// Thrown when Notion is unable to remove a shim
    ShimRemoveError {
        name: String,
    },

    ToolNotImplemented,

    /// Thrown when the shell name specified in the Notion environment is not supported.
    UnrecognizedShell {
        name: String,
    },

    /// Thrown when the postscript file was not specified in the Notion environment.
    UnspecifiedPostscript,

    /// Thrown when the shell name was not specified in the Notion environment.
    UnspecifiedShell,

    VersionParseError {
        version: String,
    },

    /// Thrown when there is an error fetching the latest version of Yarn
    YarnLatestFetchError {
        from_url: String,
    },

    /// Thrown when there is no Yarn version matching a requested semver specifier.
    YarnVersionNotFound {
        matching: String,
    },
}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorDetails::BinaryAlreadyInstalled {
                bin_name,
                existing_package,
                new_package,
            } => write!(
                f,
                "Executable '{}' is already installed by {}

Remove {} before installing {}",
                bin_name, existing_package, existing_package, new_package
            ),
            ErrorDetails::BinaryExecError => write!(f, "Could not execute command.

See `notion help install` and `notion help pin` for info about making tools available."),
            ErrorDetails::BinaryNotFound { name } => write!(f, r#"Could not find executable "{}"

Use `notion install` to add a package to your toolchain (see `notion help install` for more info)."#, name),
            ErrorDetails::CannotPinPackage { package } => {
                write!(f, "Only node and yarn can be pinned in a project

Use `npm install` or `yarn add` to select a version of {} for this project.", package)
            }
            ErrorDetails::CommandNotImplemented { command_name } => {
                write!(f, "command `{}` is not yet implemented", command_name)
            }
            ErrorDetails::CouldNotDetermineTool => write!(f, "Tool name could not be determined"),
            ErrorDetails::CreateDirError { dir } => {
                write!(f, "Could not create directory {}

Please ensure that you have the correct permissions.", dir)
            }
            ErrorDetails::DeleteDirectoryError { directory } => write!(f, "Could not remove directory
at {}

Please ensure you have correct permissions to the Notion directory.", directory),
            ErrorDetails::DeleteFileError { file } => write!(f, "Could not remove file
at {}

Please ensure you have correct permissions to the Notion directory.", file),
            ErrorDetails::DepPackageReadError => {
                write!(f, "Could not read package info for dependencies.

Please ensure that all dependencies have been installed.")
            }
            ErrorDetails::DeprecatedCommandError { command, advice } => {
                write!(f, "The subcommand `{}` is deprecated.\n{}", command, advice)
            }
            ErrorDetails::DownloadToolNetworkError { tool, from_url } => write!(
                f,
                "Could not download {}
from {}

Please verify your internet connection and ensure the correct version is specified.",
                tool, from_url
            ),
            ErrorDetails::InvalidHookCommand { command } => write!(f, "Invalid hook command: '{}'", command),
            ErrorDetails::NoBinPlatform { binary } => {
                write!(f, "Platform info for executable `{}` is missing", binary)
            }
            ErrorDetails::NodeVersionNotFound { matching } => write!(
                f,
                r#"Could not find Node version matching "{}" in the version registry.

Please verify that the version is correct."#,
                matching
            ),
            ErrorDetails::NoGlobalInstalls => write!(
                f,
                "Global package installs are not recommended.

Use `notion install` to add a package to your toolchain (see `notion help install` for more info)."
            ),
            ErrorDetails::NoHomeEnvironmentVar => {
                write!(f, "environment variable 'HOME' is not set")
            }
            ErrorDetails::NoLocalDataDir => write!(f, "Windows LocalAppData directory not found"),
            ErrorDetails::NoPackageExecutables => {
                write!(f, "Package has no binaries or executables - nothing to do")
            }
            ErrorDetails::NoPinnedNodeVersion => write!(
                f,
                "Cannot pin Yarn because the Node version is not pinned in this project.

Use `notion pin node` to pin Node first, then pin a Yarn version."
            ),
            ErrorDetails::NoPlatform => write!(
                f,
                "Could not determine Node version.

Use `notion pin node` to select a version for a project.
Use `notion install node` to select a default version."
            ),
            ErrorDetails::NoProjectYarn => write!(
                f,
                "No Yarn version found in this project.

Use `notion pin yarn` to select a version (see `notion help pin` for more info)."
            ),
            ErrorDetails::NotInPackage => write!(f, "Not in a node package"),
            ErrorDetails::NoUserYarn => write!(
                f,
                "Could not determine Yarn version.

Use `notion install yarn` to select a default version (see `notion help install for more info)."
            ),
            ErrorDetails::NoVersionsFound => write!(f, "no versions found"),
            ErrorDetails::NpxNotAvailable { version } => write!(
                f,
                "'npx' is only available with npm >= 5.2.0

This project is configured to use version {} of npm.",
                version
            ),
            // Confirming permissions is a Weak CTA in this case, but it seems the most likely error vector
            ErrorDetails::PackageInstallFailed => write!(f, "Could not install package dependencies.

Please ensure you have correct permissions to the Notion directory."),
            ErrorDetails::PackageMetadataFetchError { from_url } => write!(
                f,
                "Could not download package metadata
from {}

Please verify your internet connection.",
                from_url
            ),
            ErrorDetails::PackageParseError { file } => {
                write!(f, "Could not parse project manifest
at {}

Please ensure that the file is correctly formatted.", file)
            },
            ErrorDetails::PackageReadError { file } => {
                write!(f, "Could not read project manifest
from {}

Please ensure that the file exists.", file)
            }
            ErrorDetails::PackageUnpackError => write!(
                f,
                "Package unpack error: Could not determine unpack directory name"
            ),
            ErrorDetails::PackageVersionNotFound { name, matching } => write!(
                f,
                r#"Could not find {} version matching "{}" in the package registry.

Please verify that the version is correct."#,
                name, matching
            ),
            ErrorDetails::PathError => write!(f, "`path` internal error"),
            ErrorDetails::RegistryFetchError { tool, from_url } => write!(
                f,
                "Could not download {} version registry
from {}

Please verify your internet connection.",
                tool, from_url
            ),
            ErrorDetails::ShimCreateError { name } => write!(f, r#"Could not create shim for "{}"

Please ensure you have correct permissions to the Notion directory."#, name),
            // This case does not have a CTA as there is no avenue to allow users to remove built-in shims
            ErrorDetails::ShimRemoveBuiltInError { name } => write!(f, r#"Cannot remove built-in shim for "{}""#, name),
            ErrorDetails::ShimRemoveError { name } => write!(f, r#"Could not remove shim for "{}"

Please ensure you have correct permissions to the Notion directory."#, name),
            ErrorDetails::ToolNotImplemented => write!(f, "this tool is not yet implemented"),
            ErrorDetails::UnrecognizedShell { name } => write!(f, "Unrecognized shell: {}", name),
            ErrorDetails::UnspecifiedPostscript => {
                write!(f, "Notion postscript file not specified")
            }
            ErrorDetails::UnspecifiedShell => write!(f, "Notion shell not specified"),
            ErrorDetails::VersionParseError { version } => write!(f, r#"Could not parse version "{}"

Please verify the intended version."#, version),
            ErrorDetails::YarnLatestFetchError { from_url } => write!(
                f,
                "Could not fetch latest version of Yarn
from {}

Please verify your internet connection.",
                from_url
            ),
            ErrorDetails::YarnVersionNotFound { matching } => write!(
                f,
                r#"Could not find Yarn version matching "{}" in the version registry.

Please verify that the version is correct."#,
                matching
            ),
        }
    }
}

impl NotionFail for ErrorDetails {
    fn exit_code(&self) -> ExitCode {
        match self {
            ErrorDetails::BinaryAlreadyInstalled { .. } => ExitCode::FileSystemError,
            ErrorDetails::BinaryExecError => ExitCode::ExecutionFailure,
            ErrorDetails::BinaryNotFound { .. } => ExitCode::ExecutableNotFound,
            ErrorDetails::CannotPinPackage { .. } => ExitCode::InvalidArguments,
            ErrorDetails::CommandNotImplemented { .. } => ExitCode::NotYetImplemented,
            ErrorDetails::CouldNotDetermineTool => ExitCode::UnknownError,
            ErrorDetails::CreateDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DeleteDirectoryError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DeleteFileError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DepPackageReadError => ExitCode::FileSystemError,
            ErrorDetails::DeprecatedCommandError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            ErrorDetails::InvalidHookCommand { .. } => ExitCode::UnknownError,
            ErrorDetails::NoBinPlatform { .. } => ExitCode::ExecutionFailure,
            ErrorDetails::NodeVersionNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::NoGlobalInstalls => ExitCode::InvalidArguments,
            ErrorDetails::NoHomeEnvironmentVar => ExitCode::EnvironmentError,
            ErrorDetails::NoLocalDataDir => ExitCode::EnvironmentError,
            ErrorDetails::NoPackageExecutables { .. } => ExitCode::InvalidArguments,
            ErrorDetails::NoPinnedNodeVersion => ExitCode::ConfigurationError,
            ErrorDetails::NoPlatform => ExitCode::ConfigurationError,
            ErrorDetails::NoProjectYarn => ExitCode::ConfigurationError,
            ErrorDetails::NotInPackage => ExitCode::ConfigurationError,
            ErrorDetails::NoUserYarn => ExitCode::ConfigurationError,
            ErrorDetails::NoVersionsFound => ExitCode::NoVersionMatch,
            ErrorDetails::NpxNotAvailable { .. } => ExitCode::ExecutableNotFound,
            ErrorDetails::PackageInstallFailed => ExitCode::FileSystemError,
            ErrorDetails::PackageMetadataFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::PackageParseError { .. } => ExitCode::ConfigurationError,
            ErrorDetails::PackageReadError { .. } => ExitCode::FileSystemError,
            ErrorDetails::PackageUnpackError => ExitCode::ConfigurationError,
            ErrorDetails::PackageVersionNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::PathError => ExitCode::UnknownError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::ShimCreateError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ShimRemoveBuiltInError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::ShimRemoveError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ToolNotImplemented => ExitCode::ExecutableNotFound,
            ErrorDetails::UnrecognizedShell { .. } => ExitCode::EnvironmentError,
            ErrorDetails::UnspecifiedPostscript => ExitCode::EnvironmentError,
            ErrorDetails::UnspecifiedShell => ExitCode::EnvironmentError,
            ErrorDetails::VersionParseError { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::YarnLatestFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::YarnVersionNotFound { .. } => ExitCode::NoVersionMatch,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
