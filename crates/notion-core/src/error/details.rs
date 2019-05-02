use std::fmt;

use failure::Fail;
use textwrap::{fill, indent};

use notion_fail::{ExitCode, NotionFail};

use crate::style::display_width;
use crate::tool::ToolSpec;

const REPORT_BUG_CTA: &'static str =
    "Please rerun the command that triggered this error with the environment
variables `NOTION_DEV` set to `1` and `RUST_BACKTRACE` set to `full`, and open
an issue at https://github.com/notion-cli/notion/issues with the details!";

const PERMISSIONS_CTA: &'static str =
    "Please ensure you have correct permissions to the Notion directory.";

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

    /// Thrown when creating a temporary directory fails
    CreateTempDirError {
        in_dir: String,
    },

    /// Thrown when creating a temporary file fails
    CreateTempFileError {
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

    /// Thrown when determining the loader for a binary encountered an error
    DetermineBinaryLoaderError {
        bin: String,
    },

    DownloadToolNetworkError {
        tool: ToolSpec,
        from_url: String,
    },

    /// Thrown when building the path to an executable fails
    ExecutablePathError {
        command: String,
    },

    /// Thrown when verifying the file permissions on an executable fails
    ExecutablePermissionsError {
        bin: String,
    },

    /// Thrown when executing a hook command fails
    ExecuteHookError {
        command: String,
    },

    /// Thrown when a hook contains multiple fields (prefix, template, or bin)
    HookMultipleFieldsSpecified,

    /// Thrown when a hook doesn't contain any of the known fields (prefix, template, or bin)
    HookNoFieldsSpecified,

    InvalidHookCommand {
        command: String,
    },

    /// Thrown when output from a hook command could not be read
    InvalidHookOutput {
        command: String,
    },

    /// Thrown when a tool name is invalid per npm's rules.
    InvalidToolName {
        name: String,
        errors: Vec<String>,
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

    /// Thrown when the install dir could not be determined
    NoInstallDir,

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

    /// Thrown when unable to parse a bin config file
    ParseBinConfigError,

    /// Thrown when unable to parse the hooks.toml file
    ParseHooksError,

    /// Thrown when unable to parse the node index cache
    ParseNodeIndexCacheError,

    /// Thrown when unable to parse the node index
    ParseNodeIndexError {
        from_url: String,
    },

    /// Thrown when unable to parse the node index cache expiration
    ParseNodeIndexExpiryError,

    /// Thrown when unable to parse the npm manifest file from a node install
    ParseNpmManifestError,

    /// Thrown when unable to parse a package configuration
    ParsePackageConfigError,

    /// Thrown when unable to parse the metadata for a package
    ParsePackageMetadataError {
        from_url: String,
    },

    /// Thrown when unable to parse the platform.json file
    ParsePlatformError,

    /// Thrown when unable to parse a tool spec (`<tool>[@<version>]`)
    ParseToolSpecError {
        tool_spec: String,
    },

    /// Thrown when a publish hook contains both the url and bin fields
    PublishHookBothUrlAndBin,

    /// Thrown when a publish hook contains neither url nor bin fields
    PublishHookNeitherUrlNorBin,

    /// Thrown when there was an error reading the user bin directory
    ReadBinConfigDirError {
        dir: String,
    },

    /// Thrown when there was an error reading the config for a binary
    ReadBinConfigError {
        file: String,
    },

    /// Thrown when unable to read the default npm version file
    ReadDefaultNpmError {
        file: String,
    },

    /// Thrown when there was an error opening the hooks.toml file
    ReadHooksError {
        file: String,
    },

    /// Thrown when there was an error reading the inventory contents
    ReadInventoryDirError {
        dir: String,
    },

    /// Thrown when there was an error reading the Node Index Cache
    ReadNodeIndexCacheError {
        file: String,
    },

    /// Thrown when there was an error reading the Node Index Cache Expiration
    ReadNodeIndexExpiryError {
        file: String,
    },

    /// Thrown when there was an error reading the npm manifest file
    ReadNpmManifestError,

    /// Thrown when there was an error reading a package configuration file
    ReadPackageConfigError {
        file: String,
    },

    /// Thrown when there was an error opening the user platform file
    ReadPlatformError {
        file: String,
    },

    /// Thrown when the public registry for Node or Yarn could not be downloaded.
    RegistryFetchError {
        tool: String,
        from_url: String,
    },

    /// Thrown when there was an error copying an unpacked tool to the image directory
    SetupToolImageError {
        tool: String,
        version: String,
        dir: String,
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

    /// Thrown when serializnig a bin config to JSON fails
    StringifyBinConfigError,

    /// Thrown when serializnig a package config to JSON fails
    StringifyPackageConfigError,

    /// Thrown when serializing the platform to JSON fails
    StringifyPlatformError,

    /// Thrown when serializing the toolchain to JSON fails
    StringifyToolchainError,

    /// Thrown when a given feature has not yet been implemented
    Unimplemented {
        feature: String,
    },

    /// Thrown when unpacking an archive (tarball or zip) fails
    UnpackArchiveError {
        tool: String,
        version: String,
    },

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

    /// Thrown when there was an error writing a bin config file
    WriteBinConfigError {
        file: String,
    },

    /// Thrown when there was an error writing the default npm to file
    WriteDefaultNpmError {
        file: String,
    },

    /// Thrown when there was an error writing the node index cache
    WriteNodeIndexCacheError {
        file: String,
    },

    /// Thrown when there was an error writing the node index expiration
    WriteNodeIndexExpiryError {
        file: String,
    },

    /// Thrown when there was an error writing a package config
    WritePackageConfigError {
        file: String,
    },

    /// Thrown when there was an error writing the shasum for a package
    WritePackageShasumError {
        package: String,
        version: String,
        file: String,
    },

    /// Thrown when writing the platform.json file fails
    WritePlatformError {
        file: String,
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

{}", path, PERMISSIONS_CTA)
            }
            ErrorDetails::CouldNotDetermineTool => write!(f, "Could not determine tool name

{}", REPORT_BUG_CTA),
            ErrorDetails::CreateDirError { dir } => {
                write!(f, "Could not create directory {}

Please ensure that you have the correct permissions.", dir)
            }
            ErrorDetails::CreatePostscriptError { in_dir } => write!(f, "Could not create postscript file
in {}

{}", in_dir, PERMISSIONS_CTA),
            ErrorDetails::CreateTempDirError { in_dir } => write!(f, "Could not create temporary directory
in {}

{}", in_dir, PERMISSIONS_CTA),
            ErrorDetails::CreateTempFileError { in_dir } => write!(f, "Could not create temporary file
in {}

{}", in_dir, PERMISSIONS_CTA),
            ErrorDetails::CurrentDirError => write!(f, "Could not determine current directory

Please ensure that you have the correct permissions."),
            ErrorDetails::DeleteDirectoryError { directory } => write!(f, "Could not remove directory
at {}

{}", directory, PERMISSIONS_CTA),
            ErrorDetails::DeleteFileError { file } => write!(f, "Could not remove file
at {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::DepPackageReadError => {
                write!(f, "Could not read package info for dependencies.

Please ensure that all dependencies have been installed.")
            }
            ErrorDetails::DeprecatedCommandError { command, advice } => {
                write!(f, "The subcommand `{}` is deprecated.\n{}", command, advice)
            }
            ErrorDetails::DetermineBinaryLoaderError { bin } => write!(f, "Could not determine loader for executable '{}'

{}", bin, REPORT_BUG_CTA),
            ErrorDetails::DownloadToolNetworkError { tool, from_url } => write!(
                f,
                "Could not download {}
from {}

Please verify your internet connection and ensure the correct version is specified.",
                tool, from_url
            ),
            ErrorDetails::ExecutablePathError { command } => write!(f, "Could not determine path to executable '{}'

{}", command, REPORT_BUG_CTA),
            ErrorDetails::ExecutablePermissionsError { bin } => write!(f, "Could not verify permissions for executable '{}'

{}", bin, PERMISSIONS_CTA),
            ErrorDetails::ExecuteHookError { command } => write!(f, "Could not execute hook command: '{}'

Please ensure that the correct command is specified.", command),
            ErrorDetails::HookMultipleFieldsSpecified => write!(f, "Hook configuration includes multiple hook types.

Please include only one of 'bin', 'prefix', or 'template'"),
            ErrorDetails::HookNoFieldsSpecified => write!(f, "Hook configuration includes no hook types.

Please include one of 'bin', 'prefix', or 'template'"),
            ErrorDetails::InvalidHookCommand { command } => write!(f, "Invalid hook command: '{}'

Please ensure that the correct command is specified.", command),
            ErrorDetails::InvalidHookOutput { command } => write!(f, "Could not read output from hook command: '{}'

Please ensure that the command output is valid UTF-8 text.", command),
            ErrorDetails::InvalidToolName { name, errors } => {
                let indentation = "    ";
                let wrapped = &fill(&errors.join("\n"), display_width() - indentation.len());
                let formatted_errs = indent(&wrapped, indentation);

                let call_to_action = if errors.len() > 1 {
                    "Please fix the following errors:"
                } else {
                    "Please fix the following error:"
                };

                write!(f, "Invalid tool name `{}`

{}
{}", name, call_to_action, formatted_errs)
            },
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
            ErrorDetails::NoInstallDir => write!(f, "Could not determine Notion install directory.

Please ensure Notion was installed correctly"),
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

{}", PERMISSIONS_CTA),
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
            ErrorDetails::ParseBinConfigError => write!(f, "Could not parse executable configuration file.

{}", REPORT_BUG_CTA),
            ErrorDetails::ParseHooksError => write!(f, "Could not parse hooks.toml configuration file.

Please ensure the file is correctly formatted."),
            ErrorDetails::ParseNodeIndexCacheError => write!(f, "Could not parse Node index cache file.

{}", REPORT_BUG_CTA),
            ErrorDetails::ParseNodeIndexError { from_url } => write!(f, "Could not parse Node version index
from {}

Please verify your internet connection.", from_url),
            ErrorDetails::ParseNodeIndexExpiryError => write!(f, "Could not parse Node index cache expiration file.

{}", REPORT_BUG_CTA),
            ErrorDetails::ParseNpmManifestError => write!(f, "Could not parse package.json file for bundled npm.

Please ensure the version of Node is correct."),
            ErrorDetails::ParsePackageConfigError => write!(f, "Could not parse package configuration file.

{}", REPORT_BUG_CTA),
            ErrorDetails::ParsePackageMetadataError { from_url } => write!(f, "Could not parse package metadata
from {}

Please verify the requested package and version.", from_url),
            ErrorDetails::ParsePlatformError => write!(f, "Could not parse platform settings file.

{}", REPORT_BUG_CTA),
            ErrorDetails::ParseToolSpecError { tool_spec } => write!(f, "Could not parse tool spec `{}`

Please supply a spec in the format `<tool name>[@<version>]`.", tool_spec),
            ErrorDetails::PublishHookBothUrlAndBin => write!(f, "Publish hook configuration includes both hook types.

Please include only one of 'bin' or 'url'"),
            ErrorDetails::PublishHookNeitherUrlNorBin => write!(f, "Publish hook configuration includes no hook types.

Please include one of 'bin' or 'url'"),
            ErrorDetails::ReadBinConfigDirError { dir } => write!(f, "Could not read executable metadata directory
at {}

{}", dir, PERMISSIONS_CTA),
            ErrorDetails::ReadBinConfigError { file } => write!(f, "Could not read executable configuration
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadDefaultNpmError { file } => write!(f, "Could not read default npm version
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadHooksError { file } => write!(f, "Could not read hooks file
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadInventoryDirError { dir } => write!(f, "Could not read tool inventory contents
from {}

{}", dir, PERMISSIONS_CTA),
            ErrorDetails::ReadNodeIndexCacheError { file } => write!(f, "Could not read Node index cache
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadNodeIndexExpiryError { file } => write!(f, "Could not read Node index cache expiration
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadNpmManifestError => write!(f, "Could not read package.json file for bundled npm.

Please ensure the version of Node is correct."),
            ErrorDetails::ReadPackageConfigError { file } => write!(f, "Could not read package configuration file
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::ReadPlatformError { file } => write!(f, "Could not read default platform file
from {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::RegistryFetchError { tool, from_url } => write!(
                f,
                "Could not download {} version registry
from {}

Please verify your internet connection.",
                tool, from_url
            ),
            ErrorDetails::SetupToolImageError { tool, version, dir } => write!(f, "Could not create environment for {} v{}
at {}

{}", tool, version, dir, PERMISSIONS_CTA),
            ErrorDetails::ShimCreateError { name } => write!(f, r#"Could not create shim for "{}"

{}"#, name, PERMISSIONS_CTA),
            // This case does not have a CTA as there is no avenue to allow users to remove built-in shims
            ErrorDetails::ShimRemoveBuiltInError { name } => write!(f, r#"Cannot remove built-in shim for "{}""#, name),
            ErrorDetails::ShimRemoveError { name } => write!(f, r#"Could not remove shim for "{}"

{}"#, name, PERMISSIONS_CTA),
            ErrorDetails::StringifyBinConfigError => write!(f, "Could not serialize executable configuration.

{}", REPORT_BUG_CTA),
            ErrorDetails::StringifyPackageConfigError => write!(f, "Could not serialize package configuration.

{}", REPORT_BUG_CTA),
            ErrorDetails::StringifyPlatformError => write!(f, "Could not serialize platform settings.

{}", REPORT_BUG_CTA),
            ErrorDetails::StringifyToolchainError => write!(f, "Could not serialize toolchain settings.

{}", REPORT_BUG_CTA),
            ErrorDetails::Unimplemented { feature } => write!(f, "{} is not supported yet.", feature),
            ErrorDetails::UnpackArchiveError { tool, version } => write!(f, "Could not unpack {} v{}

Please ensure the correct version is specified.", tool, version),
            ErrorDetails::UnrecognizedShell { name } => write!(f, "Unrecognized shell '{}'

Please ensure you are using a supported shell.", name),
            ErrorDetails::UnspecifiedPostscript => {
                write!(f, "Could not determine Notion postscript file.

Please ensure Notion was installed correctly.")
            }
            ErrorDetails::UnspecifiedShell => write!(f, "Notion shell not specified"),
            ErrorDetails::VersionParseError { version } => write!(f, r#"Could not parse version "{}"

Please verify the intended version."#, version),
            ErrorDetails::WriteBinConfigError { file } => write!(f, "Could not write executable configuration
to {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::WriteDefaultNpmError { file } => write!(f, "Could not write bundled npm version
to {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::WriteNodeIndexCacheError { file } => write!(f, "Could not write Node index cache
to {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::WriteNodeIndexExpiryError { file } => write!(f, "Could not write Node index cache expiration
to {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::WritePackageConfigError { file } => write!(f, "Could not write package configuration
to {}

{}", file, PERMISSIONS_CTA),
            ErrorDetails::WritePackageShasumError { package, version, file } => write!(f, "Could not write shasum for {} v{}
to {}

{}", package, version, file, PERMISSIONS_CTA),
            ErrorDetails::WritePlatformError { file } => write!(f, "Could not save platform settings
to {}

{}", file, PERMISSIONS_CTA),
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
            ErrorDetails::CreateTempDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::CreateTempFileError { .. } => ExitCode::FileSystemError,
            ErrorDetails::CurrentDirError => ExitCode::EnvironmentError,
            ErrorDetails::DeleteDirectoryError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DeleteFileError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DepPackageReadError => ExitCode::FileSystemError,
            ErrorDetails::DeprecatedCommandError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::DetermineBinaryLoaderError { .. } => ExitCode::FileSystemError,
            ErrorDetails::DownloadToolNetworkError { .. } => ExitCode::NetworkError,
            ErrorDetails::ExecutablePathError { .. } => ExitCode::UnknownError,
            ErrorDetails::ExecutablePermissionsError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ExecuteHookError { .. } => ExitCode::ExecutionFailure,
            ErrorDetails::HookMultipleFieldsSpecified => ExitCode::ConfigurationError,
            ErrorDetails::HookNoFieldsSpecified => ExitCode::ConfigurationError,
            ErrorDetails::InvalidHookCommand { .. } => ExitCode::ExecutableNotFound,
            ErrorDetails::InvalidHookOutput { .. } => ExitCode::ExecutionFailure,
            ErrorDetails::InvalidToolName { .. } => ExitCode::InvalidArguments,
            ErrorDetails::NoBinPlatform { .. } => ExitCode::ExecutionFailure,
            ErrorDetails::NodeVersionNotFound { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::NoGlobalInstalls => ExitCode::InvalidArguments,
            ErrorDetails::NoHomeEnvironmentVar => ExitCode::EnvironmentError,
            ErrorDetails::NoInstallDir => ExitCode::EnvironmentError,
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
            ErrorDetails::ParseBinConfigError => ExitCode::UnknownError,
            ErrorDetails::ParseHooksError => ExitCode::ConfigurationError,
            ErrorDetails::ParseToolSpecError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::ParseNodeIndexCacheError => ExitCode::UnknownError,
            ErrorDetails::ParseNodeIndexError { .. } => ExitCode::NetworkError,
            ErrorDetails::ParseNodeIndexExpiryError => ExitCode::UnknownError,
            ErrorDetails::ParseNpmManifestError => ExitCode::UnknownError,
            ErrorDetails::ParsePackageConfigError => ExitCode::UnknownError,
            ErrorDetails::ParsePackageMetadataError { .. } => ExitCode::UnknownError,
            ErrorDetails::ParsePlatformError => ExitCode::ConfigurationError,
            ErrorDetails::PublishHookBothUrlAndBin => ExitCode::ConfigurationError,
            ErrorDetails::PublishHookNeitherUrlNorBin => ExitCode::ConfigurationError,
            ErrorDetails::ReadBinConfigDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadBinConfigError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadDefaultNpmError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadHooksError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadInventoryDirError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadNodeIndexCacheError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadNodeIndexExpiryError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadNpmManifestError => ExitCode::UnknownError,
            ErrorDetails::ReadPackageConfigError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ReadPlatformError { .. } => ExitCode::FileSystemError,
            ErrorDetails::RegistryFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::SetupToolImageError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ShimCreateError { .. } => ExitCode::FileSystemError,
            ErrorDetails::ShimRemoveBuiltInError { .. } => ExitCode::InvalidArguments,
            ErrorDetails::ShimRemoveError { .. } => ExitCode::FileSystemError,
            ErrorDetails::StringifyBinConfigError => ExitCode::UnknownError,
            ErrorDetails::StringifyPackageConfigError => ExitCode::UnknownError,
            ErrorDetails::StringifyPlatformError => ExitCode::UnknownError,
            ErrorDetails::StringifyToolchainError => ExitCode::UnknownError,
            ErrorDetails::Unimplemented { .. } => ExitCode::UnknownError,
            ErrorDetails::UnpackArchiveError { .. } => ExitCode::UnknownError,
            ErrorDetails::UnrecognizedShell { .. } => ExitCode::EnvironmentError,
            ErrorDetails::UnspecifiedPostscript => ExitCode::EnvironmentError,
            ErrorDetails::UnspecifiedShell => ExitCode::EnvironmentError,
            ErrorDetails::VersionParseError { .. } => ExitCode::NoVersionMatch,
            ErrorDetails::WriteBinConfigError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WriteDefaultNpmError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WriteNodeIndexCacheError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WriteNodeIndexExpiryError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WritePackageConfigError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WritePackageShasumError { .. } => ExitCode::FileSystemError,
            ErrorDetails::WritePlatformError { .. } => ExitCode::FileSystemError,
            ErrorDetails::YarnLatestFetchError { .. } => ExitCode::NetworkError,
            ErrorDetails::YarnVersionNotFound { .. } => ExitCode::NoVersionMatch,
        }
    }

    fn is_user_friendly(&self) -> bool {
        true
    }
}
