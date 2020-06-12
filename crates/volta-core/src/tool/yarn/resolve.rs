//! Provides resolution of Yarn requirements into specific versions

use std::collections::BTreeSet;

use super::super::registry_fetch_error;
use super::metadata::{RawYarnIndex, YarnIndex};
use crate::error::{Context, ErrorKind, Fallible};
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::Yarn;
use crate::version::{parse_version, VersionSpec, VersionTag};
use attohttpc::Response;
use cfg_if::cfg_if;
use log::debug;
use semver::{Version, VersionReq};

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
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None | VersionSpec::Tag(VersionTag::Latest) => resolve_latest(hooks),
        VersionSpec::Tag(tag) => Err(ErrorKind::YarnVersionNotFound {
            matching: tag.to_string(),
        }
        .into()),
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
    let releases: RawYarnIndex = attohttpc::get(&url)
        .send()
        .and_then(Response::error_for_status)
        .and_then(Response::json)
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
