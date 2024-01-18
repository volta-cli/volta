//! Provides resolution of Yarn requirements into specific versions

use super::super::registry::{
    fetch_npm_registry, public_registry_index, PackageDetails, PackageIndex,
};
use super::super::registry_fetch_error;
use super::metadata::{RawYarnIndex, YarnIndex};
use crate::error::{Context, ErrorKind, Fallible};
use crate::hook::{RegistryFormat, YarnHooks};
use crate::session::Session;
use crate::style::progress_spinner;
use crate::version::{parse_version, VersionSpec, VersionTag};
use attohttpc::Response;
use log::debug;
use node_semver::{Range, Version};

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.yarn();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None => resolve_tag(VersionTag::Latest, hooks),
        VersionSpec::Tag(tag) => resolve_tag(tag, hooks),
    }
}

fn resolve_tag(tag: VersionTag, hooks: Option<&YarnHooks>) -> Fallible<Version> {
    // This triage is complicated because we need to maintain the legacy behavior of hooks
    // First, if the tag is 'latest' and we have a 'latest' hook, we use the old behavior
    // Next, if the tag is 'latest' and we _do not_ have a 'latest' hook, we use the new behavior
    // Next, if the tag is _not_ 'latest' and we have an 'index' hook, we show an error since
    //     the previous behavior did not support generic tags
    // Finally, we don't have any relevant hooks, so we can use the new behavior
    match (tag, hooks) {
        (
            VersionTag::Latest,
            Some(&YarnHooks {
                latest: Some(ref hook),
                ..
            }),
        ) => {
            debug!("Using yarn.latest hook to determine latest-version URL");
            // does yarn3 use latest-version? no
            resolve_latest_legacy(hook.resolve("latest-version")?)
        }
        (VersionTag::Latest, _) => resolve_custom_tag(VersionTag::Latest.to_string()),
        (tag, Some(&YarnHooks { index: Some(_), .. })) => Err(ErrorKind::YarnVersionNotFound {
            matching: tag.to_string(),
        }
        .into()),
        (tag, _) => resolve_custom_tag(tag.to_string()),
    }
}

fn resolve_semver(matching: Range, hooks: Option<&YarnHooks>) -> Fallible<Version> {
    // For semver, the triage is less complicated: The previous behavior _always_ used
    // the 'index' hook, so we can check for that to decide which behavior to use.
    //
    // If the user specifies a format for the registry, we use that. Otherwise Github format
    // is the default legacy behavior.
    if let Some(&YarnHooks {
        index: Some(ref hook),
        ..
    }) = hooks
    {
        debug!("Using yarn.index hook to determine yarn index URL");
        match hook.format {
            RegistryFormat::Github => resolve_semver_legacy(matching, hook.resolve("releases")?),
            RegistryFormat::Npm => resolve_semver_npm(matching, hook.resolve("")?),
        }
    } else {
        resolve_semver_from_registry(matching)
    }
}

fn fetch_yarn_index(package: &str) -> Fallible<(String, PackageIndex)> {
    let url = public_registry_index(package);
    fetch_npm_registry(url, "Yarn")
}

fn resolve_custom_tag(tag: String) -> Fallible<Version> {
    // first try yarn2+, which uses "@yarnpkg/cli-dist" instead of "yarn"
    if let Ok((url, mut index)) = fetch_yarn_index("@yarnpkg/cli-dist") {
        if let Some(version) = index.tags.remove(&tag) {
            debug!("Found yarn@{} matching tag '{}' from {}", version, tag, url);
            if version.major == 2 {
                return Err(ErrorKind::Yarn2NotSupported.into());
            }
            return Ok(version);
        }
    }
    debug!(
        "Did not find yarn matching tag '{}' from @yarnpkg/cli-dist",
        tag
    );

    let (url, mut index) = fetch_yarn_index("yarn")?;
    match index.tags.remove(&tag) {
        Some(version) => {
            debug!("Found yarn@{} matching tag '{}' from {}", version, tag, url);
            Ok(version)
        }
        None => Err(ErrorKind::YarnVersionNotFound { matching: tag }.into()),
    }
}

fn resolve_latest_legacy(url: String) -> Fallible<Version> {
    let response_text = attohttpc::get(&url)
        .send()
        .and_then(Response::error_for_status)
        .and_then(Response::text)
        .with_context(|| ErrorKind::YarnLatestFetchError {
            from_url: url.clone(),
        })?;

    debug!("Found yarn latest version ({}) from {}", response_text, url);
    parse_version(response_text)
}

fn resolve_semver_from_registry(matching: Range) -> Fallible<Version> {
    // first try yarn2+, which uses "@yarnpkg/cli-dist" instead of "yarn"
    if let Ok((url, index)) = fetch_yarn_index("@yarnpkg/cli-dist") {
        let matching_entries: Vec<PackageDetails> = index
            .entries
            .into_iter()
            .filter(|PackageDetails { version, .. }| matching.satisfies(version))
            .collect();

        if !matching_entries.is_empty() {
            let details_opt = matching_entries
                .iter()
                .find(|PackageDetails { version, .. }| version.major >= 3);

            match details_opt {
                Some(details) => {
                    debug!(
                        "Found yarn@{} matching requirement '{}' from {}",
                        details.version, matching, url
                    );
                    return Ok(details.version.clone());
                }
                None => {
                    return Err(ErrorKind::Yarn2NotSupported.into());
                }
            }
        }
    }
    debug!(
        "Did not find yarn matching requirement '{}' for @yarnpkg/cli-dist",
        matching
    );

    let (url, index) = fetch_yarn_index("yarn")?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.satisfies(version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found yarn@{} matching requirement '{}' from {}",
                details.version, matching, url
            );
            Ok(details.version)
        }
        // at this point Yarn is not found in either registry
        None => Err(ErrorKind::YarnVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

fn resolve_semver_legacy(matching: Range, url: String) -> Fallible<Version> {
    let spinner = progress_spinner(format!("Fetching registry: {}", url));
    let releases: RawYarnIndex = attohttpc::get(&url)
        .send()
        .and_then(Response::error_for_status)
        .and_then(Response::json)
        .with_context(registry_fetch_error("Yarn", &url))?;
    let index = YarnIndex::from(releases);
    let releases = index.entries;
    spinner.finish_and_clear();
    let version_opt = releases.into_iter().rev().find(|v| matching.satisfies(v));

    match version_opt {
        Some(version) => {
            debug!(
                "Found yarn@{} matching requirement '{}' from {}",
                version, matching, url
            );
            Ok(version)
        }
        None => Err(ErrorKind::YarnVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

fn resolve_semver_npm(matching: Range, url: String) -> Fallible<Version> {
    let (url, index) = fetch_npm_registry(url, "Yarn")?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.satisfies(version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found yarn@{} matching requirement '{}' from {}",
                details.version, matching, url
            );
            Ok(details.version)
        }
        None => Err(ErrorKind::YarnVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}
