//! Provides resolution of 3rd-party packages into specific versions, using the npm repository

use super::super::registry::{PackageIndex, RawPackageMetadata};
use crate::error::{Context, ErrorKind, Fallible, VoltaError};
use crate::hook::ToolHooks;
use crate::platform::CliPlatform;
use crate::run::{self, ToolCommand};
use crate::session::Session;
use crate::style::{progress_spinner, tool_version};
use crate::tool::PackageDetails;
use crate::version::{VersionSpec, VersionTag};
use log::debug;
use reqwest::blocking::Response;
use semver::VersionReq;

pub fn resolve(
    name: &str,
    matching: VersionSpec,
    session: &mut Session,
) -> Fallible<PackageDetails> {
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(name, requirement, session),
        VersionSpec::Exact(version) => resolve_semver(name, VersionReq::exact(&version), session),
        VersionSpec::None | VersionSpec::Tag(VersionTag::Latest) => {
            resolve_tag(name, "latest", session)
        }
        VersionSpec::Tag(tag) => resolve_tag(name, &tag.to_string(), session),
    }
}

fn resolve_tag(name: &str, tag: &str, session: &mut Session) -> Fallible<PackageDetails> {
    let package_index = match session.hooks()?.package() {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using packages.index hook to determine package metadata URL");
            let url = hook.resolve(&name)?;
            resolve_package_metadata(name, &url)?
        }
        _ => npm_view_query(name, tag, session)?,
    };

    let mut entries = package_index.entries.into_iter();
    let details_opt = package_index
        .tags
        .get(tag)
        .and_then(|v| entries.find(|PackageDetails { version, .. }| v == version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found {} latest version ({}) from {}",
                name, details.version, details.tarball_url
            );
            Ok(details)
        }
        None => Err(ErrorKind::PackageVersionNotFound {
            name: name.to_string(),
            matching: tag.into(),
        }
        .into()),
    }
}

fn resolve_semver(
    name: &str,
    matching: VersionReq,
    session: &mut Session,
) -> Fallible<PackageDetails> {
    let package_index = match session.hooks()?.package() {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using packages.index hook to determine package metadata URL");
            let url = hook.resolve(&name)?;
            resolve_package_metadata(name, &url)?
        }
        _ => npm_view_query(name, &matching.to_string(), session)?,
    };

    let details_opt = package_index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.matches(&version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found {}@{} matching requirement '{}' from {}",
                name, details.version, matching, details.tarball_url
            );
            Ok(details)
        }
        None => Err(ErrorKind::PackageVersionNotFound {
            name: name.to_string(),
            matching: matching.to_string(),
        }
        .into()),
    }
}

/// Use `npm view` to get the info for the package. This supports:
///
/// * normal package installation from the public npm repo
/// * installing packages from alternate registries, configured via .npmrc
fn npm_view_query(name: &str, version: &str, session: &mut Session) -> Fallible<PackageIndex> {
    let command = npm_view_command_for(name, version, session)?;
    debug!("Running command: `{:?}`", command);

    let spinner = progress_spinner(&format!(
        "Querying metadata for {}",
        tool_version(name, version)
    ));
    let output = command.output()?;
    spinner.finish_and_clear();

    if !output.status.success() {
        debug!(
            "Command failed, stderr is:\n{}",
            String::from_utf8_lossy(&output.stderr).to_string()
        );
        debug!("Exit code is {:?}", output.status.code());
        return Err(ErrorKind::NpmViewMetadataFetchError {
            package: name.to_string(),
        }
        .into());
    }

    let response_json = String::from_utf8_lossy(&output.stdout);

    // Sometimes the returned JSON is an array (semver case), otherwise it's a single object.
    // Check if the first char is '[' and parse as an array if so
    if response_json.starts_with('[') {
        let metadatas: Vec<super::metadata::NpmViewData> = serde_json::de::from_str(&response_json)
            .with_context(|| ErrorKind::NpmViewMetadataParseError {
                package: name.to_string(),
            })?;
        debug!("[parsed package metadata (array)]\n{:?}", metadatas);

        // get latest version, making sure the array is not empty
        let tags = match metadatas.get(0) {
            Some(m) => m.dist_tags.clone(),
            None => {
                return Err(ErrorKind::PackageNotFound {
                    package: name.to_string(),
                }
                .into())
            }
        };

        let mut entries: Vec<PackageDetails> = metadatas.into_iter().map(|e| e.into()).collect();
        // sort so that the versions are ordered highest-to-lowest
        entries.sort_by(|a, b| b.version.cmp(&a.version));

        debug!("[sorted entries]\n{:?}", entries);

        Ok(PackageIndex { tags, entries })
    } else {
        let metadata: super::metadata::NpmViewData = serde_json::de::from_str(&response_json)
            .with_context(|| ErrorKind::NpmViewMetadataParseError {
                package: name.to_string(),
            })?;
        debug!("[parsed package metadata (single)]\n{:?}", metadata);

        Ok(PackageIndex {
            tags: metadata.dist_tags.clone(),
            entries: vec![metadata.into()],
        })
    }
}

// build a command to run `npm view` with json output
fn npm_view_command_for(name: &str, version: &str, session: &mut Session) -> Fallible<ToolCommand> {
    let mut command = run::npm::command(CliPlatform::default(), session)?;
    command.arg("view");
    command.arg("--json");
    command.arg(format!("{}@{}", name, version));
    Ok(command)
}

// fetch metadata for the input url
fn resolve_package_metadata(package_name: &str, package_info_url: &str) -> Fallible<PackageIndex> {
    let spinner = progress_spinner(&format!("Fetching package metadata: {}", package_info_url));
    let response_text = reqwest::blocking::get(package_info_url)
        .and_then(Response::error_for_status)
        .and_then(Response::text)
        .map_err(|err| {
            let kind = match err.status() {
                Some(reqwest::StatusCode::NOT_FOUND) => ErrorKind::PackageNotFound {
                    package: package_name.into(),
                },
                _ => ErrorKind::PackageMetadataFetchError {
                    from_url: package_info_url.into(),
                },
            };

            VoltaError::from_source(err, kind)
        })?;

    let metadata: RawPackageMetadata =
        serde_json::de::from_str(&response_text).with_context(|| {
            ErrorKind::ParsePackageMetadataError {
                from_url: package_info_url.to_string(),
            }
        })?;

    spinner.finish_and_clear();
    Ok(metadata.into())
}
