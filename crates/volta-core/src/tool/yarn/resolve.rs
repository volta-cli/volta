//! Provides resolution of Yarn requirements into specific versions

use super::super::registry::{
    public_registry_index, PackageDetails, PackageIndex, RawPackageMetadata,
    NPM_ABBREVIATED_ACCEPT_HEADER,
};
use super::super::registry_fetch_error;
use super::metadata::{RawYarnIndex, YarnIndex};
use crate::error::{Context, ErrorKind, Fallible};
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::Yarn;
use crate::version::{parse_version, VersionSpec, VersionTag};
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use semver::{Version, VersionReq};

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.yarn();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None => resolve_tag(VersionTag::Latest, hooks),
        VersionSpec::Tag(tag) => resolve_tag(tag, hooks),
    }
}

fn resolve_tag(tag: VersionTag, hooks: Option<&ToolHooks<Yarn>>) -> Fallible<Version> {
    // This triage is complicated because we need to maintain the legacy behavior of hooks
    // First, if the tag is 'latest' and we have a 'latest' hook, we use the old behavior
    // Next, if the tag is 'latest' and we _do not_ have a 'latest' hook, we use the new behavior
    // Next, if the tag is _not_ 'latest' and we have an 'index' hook, we show an error since
    //     the previous behavior did not support generic tags
    // Finally, we don't have any relevant hooks, so we can use the new behavior
    match (tag, hooks) {
        (
            VersionTag::Latest,
            Some(&ToolHooks {
                latest: Some(ref hook),
                ..
            }),
        ) => {
            debug!("Using yarn.latest hook to determine latest-version URL");
            resolve_latest_legacy(hook.resolve("latest-version")?)
        }
        (VersionTag::Latest, _) => resolve_custom_tag(VersionTag::Latest.to_string()),
        (tag, Some(&ToolHooks { index: Some(_), .. })) => Err(ErrorKind::YarnVersionNotFound {
            matching: tag.to_string(),
        }
        .into()),
        (tag, _) => resolve_custom_tag(tag.to_string()),
    }
}

fn resolve_semver(matching: VersionReq, hooks: Option<&ToolHooks<Yarn>>) -> Fallible<Version> {
    // For semver, the triage is less complicated: The previous behavior _always_ used
    // the 'index' hook, so we can check for that to decide which behavior to use.
    if let Some(&ToolHooks {
        index: Some(ref hook),
        ..
    }) = hooks
    {
        debug!("Using yarn.index hook to determine yarn index URL");
        resolve_semver_legacy(matching, hook.resolve("releases")?)
    } else {
        resolve_semver_from_registry(matching)
    }
}

fn fetch_yarn_index() -> Fallible<(String, PackageIndex)> {
    let url = public_registry_index("yarn");
    let spinner = progress_spinner(&format!("Fetching public registry: {}", url));
    let http_client = Client::new();
    let metadata: RawPackageMetadata = http_client
        .get(&url)
        .header(ACCEPT, NPM_ABBREVIATED_ACCEPT_HEADER)
        .send()
        .and_then(|resp| resp.json())
        .with_context(registry_fetch_error("Yarn", &url))?;

    spinner.finish_and_clear();
    Ok((url, metadata.into()))
}

fn resolve_custom_tag(tag: String) -> Fallible<Version> {
    let (url, mut index) = fetch_yarn_index()?;

    match index.tags.remove(&tag) {
        Some(version) => {
            debug!("Found yarn@{} matching tag '{}' from {}", version, tag, url);
            Ok(version)
        }
        None => Err(ErrorKind::YarnVersionNotFound { matching: tag }.into()),
    }
}

fn resolve_latest_legacy(url: String) -> Fallible<Version> {
    let response_text = reqwest::blocking::get(&url)
        .and_then(|resp| resp.text())
        .with_context(|| ErrorKind::YarnLatestFetchError {
            from_url: url.clone(),
        })?;

    debug!("Found yarn latest version ({}) from {}", response_text, url);
    parse_version(response_text)
}

fn resolve_semver_from_registry(matching: VersionReq) -> Fallible<Version> {
    let (url, index) = fetch_yarn_index()?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.matches(&version));

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

fn resolve_semver_legacy(matching: VersionReq, url: String) -> Fallible<Version> {
    let spinner = progress_spinner(&format!("Fetching public registry: {}", url));
    let releases: RawYarnIndex = reqwest::blocking::get(&url)
        .and_then(|resp| resp.json())
        .with_context(registry_fetch_error("Yarn", &url))?;
    let index = YarnIndex::from(releases);
    let releases = index.entries;
    spinner.finish_and_clear();
    let version_opt = releases.into_iter().rev().find(|v| matching.matches(v));

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
