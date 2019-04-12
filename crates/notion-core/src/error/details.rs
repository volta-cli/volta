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

    /// Thrown when building the virtual environment path fails
    BuildPathError,

    /// Thrown when a user tries to `notion pin` something other than node/yarn/npm.
    CannotPinPackage {
        package: String,
    },

    /// Thrown when the Completions out-dir is not a directory
    CompletionsOutDirError,

    /// Thrown when the containing directory could not be determined
    ContainingDirError {
        path: String,
    },

    CouldNotDetermineTool,

    CreateDirError {
        dir: String,
    },

    /// Thrown when unable to create the postscript file
    CreatePostscriptError {
        in_dir: String,
    },

    CurrentDirError,

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

    /// Thrown when writing a package manifest fails
    PackageWriteError {
        file: String,
    },

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

    /// Thrown when serializing the toolchain to JSON fails
    StringifyToolchainError,

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

Please remove {} before installing {}",
                bin_name, existing_package, existing_package, new_package
            ),
            ErrorDetails::BinaryExecError => write!(f, "Could not execute command.

See `notion help install` and `notion help pin` for info about making tools available."),
            ErrorDetails::BinaryNotFound { name } => write!(f, r#"Could not find executable "{}"

Use `notion install` to add a package to your toolchain (see `notion help install` for more info)."#, name),
            ErrorDetails::BuildPathError => {
                write!(f, "Could not create execution environment.

Please ensure your PATH is valid.")
            }
            ErrorDetails::CannotPinPackage { package } => {
                write!(f, "Only node and yarn can be pinned in a project

Use `npm install` or `yarn add` to select a version of {} for this project.", package)
            }
            ErrorDetails::CompletionsOutDirError => {
                write!(f, "out-dir must be a directory.

Please ensure the directory exists and that you have correct permissions.")
            }
            ErrorDetails::ContainingDirError { path } => {
                write!(f, "Could not determine directory information
for {}

Please ensure you have correct permissions to the Notion directory.", path)
            }
            // No CTA as there is no path to fixing not being able to determine the tool name
            ErrorDetails::CouldNotDetermineTool => write!(f, "Could not determine tool name"),
            ErrorDetails::CreateDirError { dir } => {
                write!(f, "Could not create directory {}

Please ensure that you have the correct permissions.", dir)
            }
            ErrorDetails::CreatePostscriptError { in_dir } => write!(f, "Could not create postscript file
in {}

Please ensure you have correct permissions to the Notion directory.", in_dir),
            ErrorDetails::CurrentDirError => write!(f, "Could not determine current directory

Please ensure that you have the correct permissions."),
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
            ErrorDetails::InvalidHookCommand { command } => write!(f, "Invalid hook command: '{}'

Please ensure that the correct command is specified.", command),
            ErrorDetails::NoBinPlatform { binary } => {
                write!(f, "Platform info for executable `{}` is missing

Please uninstall and re-install the package that provides that executable.", binary)
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
                write!(f, "Could not determine home directory.

Please ensure the environment variable 'HOME' is set.")
            }
            ErrorDetails::NoLocalDataDir => write!(f, "Could not determine LocalAppData directory.

Please ensure the directory is available."),
            ErrorDetails::NoPackageExecutables => {
                write!(f, "Package has no executables to install.

Please verify the intended package name.")
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
            ErrorDetails::NotInPackage => write!(f, "Not in a node package.

Use `notion install` to select a default version of a tool."),
            ErrorDetails::NoUserYarn => write!(
                f,
                "Could not determine Yarn version.

Use `notion install yarn` to select a default version (see `notion help install for more info)."
            ),
            // No CTA as this error is purely informational
            ErrorDetails::NoVersionsFound => write!(f, "No tool versions found"),
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
                "Could not determine package directory layout.

Please ensure the package is correctly formatted."
            ),
            ErrorDetails::PackageVersionNotFound { name, matching } => write!(
                f,
                r#"Could not find {} version matching "{}" in the package registry.

Please verify that the version is correct."#,
                name, matching
            ),
            ErrorDetails::PackageWriteError { file } => write!(f, "Could not write project manifest
to {}

Please ensure you have correct permissions.", file),
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
            // Note: No CTA as this is a purely internal operation and serializing should not fail
            ErrorDetails::StringifyToolchainError => write!(f, "Could not serialize toolchain settings."),
            ErrorDetails::UnrecognizedShell { name } => write!(f, "Unrecognized shell '{}'

Please ensure you are using a supported shell.", name),
            ErrorDetails::UnspecifiedPostscript => {
                write!(f, "Could not determine Notion postscript file.

Please ensure Notion was installed correctly.")
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
            ErrorDetails::BuildPathError => ExitCode::EnvironmentError,
            ErrorDetails::CannotPinPackage { .. } => ExitCode::InvalidArguments,
            ErrorDetails::CompletionsOutDirError => ExitCode::InvalidArguments,
            ErrorDetails::ContainingDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::CouldNotDetermineTool => ExitCode::UnknownError,
            ErrorDetails::CreateDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::CreatePostscriptError { .. } => ExitCode::FileSystemError,
            ErrorDetails::CurrentDirError => ExitCode::EnvironmentError,
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
            ErrorDetails::PackageWriteError { .. } => ExitCode::FileSystemError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::ShimCreateError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ShimRemoveBuiltInError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::ShimRemoveError { .. } => ExitCode::FileSystemError,
            ErrorDetails::StringifyToolchainError => ExitCode::UnknownError,
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
