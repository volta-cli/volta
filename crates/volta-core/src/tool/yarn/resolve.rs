//! Provides resolution of Yarn requirements into specific versions

use std::collections::BTreeSet;

use super::super::registry_fetch_error;
use super::serial;
use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::Yarn;
use crate::version::VersionSpec;
use cfg_if::cfg_if;
use log::debug;
use semver::{Version, VersionReq};
use volta_fail::{Fallible, ResultExt};

// ISSUE (#86): Move public repository URLs to config file
cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_yarn_version_index() -> String {
            format!("{}/yarn-releases/index.json", mockito::SERVER_URL)
        }
        fn public_yarn_latest_version() -> String {
            format!("{}/yarn-latest", mockito::SERVER_URL)
        }
    } else {
        /// Return the URL of the index of available Yarn versions on the public git repository.
        fn public_yarn_version_index() -> String {
            "https://api.github.com/repos/yarnpkg/yarn/releases".to_string()
        }
        /// URL of the latest Yarn version on the public yarnpkg.com
        fn public_yarn_latest_version() -> String {
            "https://yarnpkg.com/latest-version".to_string()
        }
    }
}

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.yarn();
    match matching {
        VersionSpec::Latest | VersionSpec::Lts => resolve_latest(hooks),
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
    }
}

fn resolve_latest(hooks: Option<&ToolHooks<Yarn>>) -> Fallible<Version> {
    let url = match hooks {
        Some(&ToolHooks {
            latest: Some(ref hook),
            ..
        }) => {
            debug!("Using yarn.latest hook to determine latest-version URL");
            hook.resolve("latest-version")?
        }
        _ => public_yarn_latest_version(),
    };
    let response_text = reqwest::get(&url)
        .and_then(|mut resp| resp.text())
        .with_context(|_| ErrorDetails::YarnLatestFetchError {
            from_url: url.clone(),
        })?;

    debug!("Found yarn latest version ({}) from {}", response_text, url);
    VersionSpec::parse_version(response_text)
}

fn resolve_semver(matching: VersionReq, hooks: Option<&ToolHooks<Yarn>>) -> Fallible<Version> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using yarn.index hook to determine yarn index URL");
            hook.resolve("releases")?
        }
        _ => public_yarn_version_index(),
    };

    let spinner = progress_spinner(&format!("Fetching public registry: {}", url));
    let releases: serial::RawYarnIndex = reqwest::get(&url)
        .and_then(|mut resp| resp.json())
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
        None => Err(ErrorDetails::YarnVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

/// The public Yarn index.
pub struct YarnIndex {
    pub(super) entries: BTreeSet<Version>,
}
