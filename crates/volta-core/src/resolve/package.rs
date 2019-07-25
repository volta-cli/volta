use std::process::Command;

use crate::command::create_command;
use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::{progress_spinner, tool_version};
use crate::tool::{Package, PackageDetails};
use crate::version::VersionSpec;
use log::debug;
use semver::{Version, VersionReq};
use volta_fail::{throw, Fallible, ResultExt};

pub fn resolve(
    name: &str,
    matching: VersionSpec,
    session: &mut Session,
) -> Fallible<PackageDetails> {
    let hooks = session.hooks()?.package();
    match matching {
        VersionSpec::Latest | VersionSpec::Lts => resolve_latest(name, hooks),
        VersionSpec::Semver(requirement) => resolve_semver(name, requirement, hooks),
        VersionSpec::Exact(version) => resolve_semver(name, VersionReq::exact(&version), hooks),
    }
}

fn resolve_latest(name: &str, hooks: Option<&ToolHooks<Package>>) -> Fallible<PackageDetails> {
    let package_index = match hooks {
        Some(&ToolHooks {
            latest: Some(ref hook),
            ..
        }) => {
            debug!("Using packages.latest hook to determine package metadata URL");
            let url = hook.resolve(&name)?;
            resolve_package_metadata(name, &url)?.into()
        }
        _ => npm_view_query(name, "latest")?,
    };

    let latest = package_index.latest.clone();

    let details_opt = match_package_details(package_index, |PackageDetails { version, .. }| {
        &latest == version
    });

    match details_opt {
        Some(details) => {
            debug!(
                "Found {} latest version ({}) from {}",
                name, details.version, details.tarball_url
            );
            Ok(details)
        }
        None => Err(ErrorDetails::PackageVersionNotFound {
            name: name.to_string(),
            matching: "latest".into(),
        }
        .into()),
    }
}

fn resolve_semver(
    name: &str,
    matching: VersionReq,
    hooks: Option<&ToolHooks<Package>>,
) -> Fallible<PackageDetails> {
    let package_index = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using packages.index hook to determine package metadata URL");
            let url = hook.resolve(&name)?;
            resolve_package_metadata(name, &url)?.into()
        }
        _ => npm_view_query(name, &matching.to_string())?,
    };

    let details_opt = match_package_details(package_index, |PackageDetails { version, .. }| {
        matching.matches(&version)
    });

    match details_opt {
        Some(details) => {
            debug!(
                "Found {}@{} matching requirement '{}' from {}",
                name, details.version, matching, details.tarball_url
            );
            Ok(details)
        }
        None => Err(ErrorDetails::PackageVersionNotFound {
            name: name.to_string(),
            matching: matching.to_string(),
        }
        .into()),
    }
}

/// Index of versions of a specific package.
pub struct PackageIndex {
    pub latest: Version,
    pub entries: Vec<PackageDetails>,
}

// use the input predicate to match a package in the index
fn match_package_details(
    index: PackageIndex,
    predicate: impl Fn(&PackageDetails) -> bool,
) -> Option<PackageDetails> {
    let mut entries = index.entries.into_iter();
    entries.find(predicate)
}

/// Use `npm view` to get the info for the package. This supports:
///
/// * normal package installation from the public npm repo
/// * installing packages from alternate registries, configured via .npmrc
fn npm_view_query(name: &str, version: &str) -> Fallible<PackageIndex> {
    let mut command = npm_view_command_for(name, version);
    debug!("Running command: `{:?}`", command);

    let spinner = progress_spinner(&format!(
        "Querying metadata for {}",
        tool_version(name, version)
    ));
    let output = command
        .output()
        .with_context(|_| ErrorDetails::NpmViewError)?;
    spinner.finish_and_clear();

    if !output.status.success() {
        debug!(
            "Command failed, stderr is:\n{}",
            String::from_utf8_lossy(&output.stderr).to_string()
        );
        debug!("Exit code is {:?}", output.status.code());
        throw!(ErrorDetails::NpmViewMetadataFetchError);
    }

    let response_json = String::from_utf8_lossy(&output.stdout);

    // Sometimes the returned JSON is an array (semver case), otherwise it's a single object.
    // Check if the first char is '[' and parse as an array if so
    if response_json.chars().next() == Some('[') {
        let metadatas: Vec<super::serial::NpmViewData> =
            serde_json::de::from_str(&response_json)
                .with_context(|_| ErrorDetails::NpmViewMetadataParseError)?;
        debug!("[parsed package metadata (array)]\n{:?}", metadatas);

        // get latest version, making sure the array is not empty
        let latest = match metadatas.iter().next() {
            Some(m) => m.dist_tags.latest.clone(),
            None => throw!(ErrorDetails::PackageNotFound {
                package: name.to_string()
            }),
        };

        let mut entries: Vec<PackageDetails> = metadatas.into_iter().map(|e| e.into()).collect();
        // sort so that the versions are ordered highest-to-lowest
        entries.sort_by(|a, b| b.version.cmp(&a.version));

        debug!("[sorted entries]\n{:?}", entries);

        Ok(PackageIndex { latest, entries })
    } else {
        let metadata: super::serial::NpmViewData = serde_json::de::from_str(&response_json)
            .with_context(|_| ErrorDetails::NpmViewMetadataParseError)?;
        debug!("[parsed package metadata (single)]\n{:?}", metadata);

        Ok(PackageIndex {
            latest: metadata.dist_tags.latest.clone(),
            entries: vec![metadata.into()],
        })
    }
}

// build a command to run `npm view` with json output
fn npm_view_command_for(name: &str, version: &str) -> Command {
    let mut command = create_command("npm");
    command.args(&["view", "--json", &format!("{}@{}", name, version)]);
    command
}

// fetch metadata for the input url
fn resolve_package_metadata(
    package_name: &str,
    package_info_url: &str,
) -> Fallible<super::serial::RawPackageMetadata> {
    let spinner = progress_spinner(&format!("Fetching package metadata: {}", package_info_url));
    let response_text = reqwest::get(package_info_url)
        .and_then(|resp| resp.error_for_status())
        .and_then(|mut resp| resp.text())
        .with_context(|err| match err.status() {
            Some(reqwest::StatusCode::NOT_FOUND) => ErrorDetails::PackageNotFound {
                package: package_name.into(),
            },
            _ => ErrorDetails::PackageMetadataFetchError {
                from_url: package_info_url.into(),
            },
        })?;

    let metadata: super::serial::RawPackageMetadata = serde_json::de::from_str(&response_text)
        .with_context(|_| ErrorDetails::ParsePackageMetadataError {
            from_url: package_info_url.to_string(),
        })?;

    spinner.finish_and_clear();
    Ok(metadata)
}
