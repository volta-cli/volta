//! Provides resolution of npm Version requirements into specific versions

use super::super::registry_fetch_error;
use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::package;
use crate::tool::Npm;
use crate::version::{VersionSpec, VersionTag};
use cfg_if::cfg_if;
use log::debug;
use reqwest::header::ACCEPT;
use reqwest::Client;
use semver::{Version, VersionReq};
use volta_fail::{Fallible, ResultExt};

// Accept header needed to request the abbreviated metadata from the npm registry
// See https://github.com/npm/registry/blob/master/docs/responses/package-metadata.md
static NPM_ABBREVIATED_ACCEPT_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_npm_version_index() -> String {
            format!("{}/npm", mockito::SERVER_URL)
        }
    } else {
        fn public_npm_version_index() -> String {
            "https://registry.npmjs.org/npm".to_string()
        }
    }
}

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.npm();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None | VersionSpec::Tag(VersionTag::Latest) => resolve_tag("latest", hooks),
        VersionSpec::Tag(tag) => resolve_tag(&tag.to_string(), hooks),
    }
}

fn fetch_npm_index(
    hooks: Option<&ToolHooks<Npm>>,
) -> Fallible<(String, package::resolve::PackageIndex)> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using npm.index hook to determine npm index URL");
            hook.resolve("npm")?
        }
        _ => public_npm_version_index(),
    };

    let spinner = progress_spinner(&format!("Fetching public registry: {}", url));
    let http_client = Client::new();
    let metadata: package::serial::RawPackageMetadata = http_client
        .get(&url)
        .header(ACCEPT, NPM_ABBREVIATED_ACCEPT_HEADER)
        .send()
        .and_then(|mut resp| resp.json())
        .with_context(registry_fetch_error("npm", &url))?;

    spinner.finish_and_clear();
    Ok((url, metadata.into()))
}

fn resolve_tag(tag: &str, hooks: Option<&ToolHooks<Npm>>) -> Fallible<Version> {
    let (url, mut index) = fetch_npm_index(hooks)?;

    match index.tags.remove(tag) {
        Some(version) => {
            debug!("Found npm@{} matching tag '{}' from {}", version, tag, url);
            Ok(version)
        }
        None => Err(ErrorDetails::NpmVersionNotFound {
            matching: tag.into(),
        }
        .into()),
    }
}

fn resolve_semver(matching: VersionReq, hooks: Option<&ToolHooks<Npm>>) -> Fallible<Version> {
    let (url, index) = fetch_npm_index(hooks)?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|package::PackageDetails { version, .. }| matching.matches(&version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found npm@{} matching requirement '{}' from {}",
                details.version, matching, url
            );
            Ok(details.version)
        }
        None => Err(ErrorDetails::NpmVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}
