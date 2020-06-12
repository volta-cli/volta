//! Provides resolution of Node requirements into specific versions, using the NodeJS index

use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use super::super::registry_fetch_error;
use super::metadata::{NodeEntry, NodeIndex, RawNodeIndex};
use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::{create_staging_file, read_file};
use crate::hook::ToolHooks;
use crate::layout::volta_home;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::tool::Node;
use crate::version::{VersionSpec, VersionTag};
use attohttpc::Response;
use cfg_if::cfg_if;
use fs_utils::ensure_containing_dir_exists;
use hyperx::header::{CacheControl, CacheDirective, Expires, HttpDate, TypedHeaders};
use log::debug;
use semver::{Version, VersionReq};

// ISSUE (#86): Move public repository URLs to config file
cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_node_version_index() -> String {
            format!("{}/node-dist/index.json", mockito::SERVER_URL)
        }
    } else {
        /// Returns the URL of the index of available Node versions on the public Node server.
        fn public_node_version_index() -> String {
            "https://nodejs.org/dist/index.json".to_string()
        }
    }
}

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.node();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None | VersionSpec::Tag(VersionTag::Lts) => resolve_lts(hooks),
        VersionSpec::Tag(VersionTag::Latest) => resolve_latest(hooks),
        VersionSpec::Tag(VersionTag::LtsRequirement(req)) => resolve_lts_semver(req, hooks),
        // Node doesn't have "tagged" versions (apart from 'latest' and 'lts'), so custom tags will always be an error
        VersionSpec::Tag(VersionTag::Custom(tag)) => {
            Err(ErrorKind::NodeVersionNotFound { matching: tag }.into())
        }
    }
}

fn resolve_latest(hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    // NOTE: This assumes the registry always produces a list in sorted order
    //       from newest to oldest. This should be specified as a requirement
    //       when we document the plugin API.
    let url = match hooks {
        Some(&ToolHooks {
            latest: Some(ref hook),
            ..
        }) => {
            debug!("Using node.latest hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt = match_node_version(&url, |_| true)?;

    match version_opt {
        Some(version) => {
            debug!("Found latest node version ({}) from {}", version, url);
            Ok(version)
        }
        None => Err(ErrorKind::NodeVersionNotFound {
            matching: "latest".into(),
        }
        .into()),
    }
}

fn resolve_lts(hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using node.index hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt = match_node_version(&url, |&NodeEntry { lts, .. }| lts)?;

    match version_opt {
        Some(version) => {
            debug!("Found newest LTS node version ({}) from {}", version, url);
            Ok(version)
        }
        None => Err(ErrorKind::NodeVersionNotFound {
            matching: "lts".into(),
        }
        .into()),
    }
}

fn resolve_semver(matching: VersionReq, hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    // ISSUE #34: also make sure this OS is available for this version
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using node.index hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };
    let version_opt =
        match_node_version(&url, |NodeEntry { version, .. }| matching.matches(version))?;

    match version_opt {
        Some(version) => {
            debug!(
                "Found node@{} matching requirement '{}' from {}",
                version, matching, url
            );
            Ok(version)
        }
        None => Err(ErrorKind::NodeVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

fn resolve_lts_semver(matching: VersionReq, hooks: Option<&ToolHooks<Node>>) -> Fallible<Version> {
    // ISSUE #34: also make sure this OS is available for this version
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using node.index hook to determine node index URL");
            hook.resolve("index.json")?
        }
        _ => public_node_version_index(),
    };

    let first_pass = match_node_version(
        &url,
        |&NodeEntry {
             ref version, lts, ..
         }| { lts && matching.matches(version) },
    )?;

    match first_pass {
        Some(version) => {
            debug!(
                "Found LTS node@{} matching requirement '{}' from {}",
                version, matching, url
            );
            return Ok(version);
        }
        None => debug!(
            "No LTS version found matching requirement '{}', checking for non-LTS",
            matching
        ),
    };

    match match_node_version(&url, |NodeEntry { version, .. }| matching.matches(version))? {
        Some(version) => {
            debug!(
                "Found non-LTS node@{} matching requirement '{}' from {}",
                version, matching, url
            );
            Ok(version)
        }
        None => Err(ErrorKind::NodeVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

fn match_node_version(
    url: &str,
    predicate: impl Fn(&NodeEntry) -> bool,
) -> Fallible<Option<Version>> {
    let index: NodeIndex = resolve_node_versions(url)?.into();
    let mut entries = index.entries.into_iter();
    Ok(entries
        .find(predicate)
        .map(|NodeEntry { version, .. }| version))
}

/// Reads a public index from the Node cache, if it exists and hasn't expired.
fn read_cached_opt(url: &str) -> Fallible<Option<RawNodeIndex>> {
    let expiry_file = volta_home()?.node_index_expiry_file();
    let expiry = read_file(&expiry_file).with_context(|| ErrorKind::ReadNodeIndexExpiryError {
        file: expiry_file.to_owned(),
    })?;

    if let Some(date) = expiry {
        let expiry_date =
            HttpDate::from_str(&date).with_context(|| ErrorKind::ParseNodeIndexExpiryError)?;
        let current_date = HttpDate::from(SystemTime::now());

        if current_date < expiry_date {
            let index_file = volta_home()?.node_index_file();
            let cached =
                read_file(&index_file).with_context(|| ErrorKind::ReadNodeIndexCacheError {
                    file: index_file.to_owned(),
                })?;

            if let Some(content) = cached {
                if content.starts_with(url) {
                    return serde_json::de::from_str(&content[url.len()..])
                        .with_context(|| ErrorKind::ParseNodeIndexCacheError);
                }
            }
        }
    }

    Ok(None)
}

/// Get the cache max-age of an HTTP reponse.
fn max_age(headers: &attohttpc::header::HeaderMap) -> u32 {
    if let Ok(cache_control_header) = headers.decode::<CacheControl>() {
        for cache_directive in cache_control_header.iter() {
            if let CacheDirective::MaxAge(max_age) = cache_directive {
                return *max_age;
            }
        }
    }

    // Default to four hours.
    4 * 60 * 60
}

fn resolve_node_versions(url: &str) -> Fallible<RawNodeIndex> {
    match read_cached_opt(url)? {
        Some(serial) => {
            debug!("Found valid cache of Node version index");
            Ok(serial)
        }
        None => {
            debug!("Node index cache was not found or was invalid");
            let spinner = progress_spinner(&format!("Fetching public registry: {}", url));

            let (_, headers, response) = attohttpc::get(url)
                .send()
                .and_then(Response::error_for_status)
                .with_context(registry_fetch_error("Node", url))?
                .split();

            let response_text = response
                .text()
                .with_context(registry_fetch_error("Node", url))?;

            let index: RawNodeIndex =
                serde_json::de::from_str(&response_text).with_context(|| {
                    ErrorKind::ParseNodeIndexError {
                        from_url: url.to_string(),
                    }
                })?;

            let cached = create_staging_file()?;

            let mut cached_file: &File = cached.as_file();
            writeln!(cached_file, "{}", url)
                .and_then(|_| cached_file.write(response_text.as_bytes()))
                .with_context(|| ErrorKind::WriteNodeIndexCacheError {
                    file: cached.path().to_path_buf(),
                })?;

            let index_cache_file = volta_home()?.node_index_file();
            ensure_containing_dir_exists(&index_cache_file).with_context(|| {
                ErrorKind::ContainingDirError {
                    path: index_cache_file.to_owned(),
                }
            })?;
            cached.persist(&index_cache_file).with_context(|| {
                ErrorKind::WriteNodeIndexCacheError {
                    file: index_cache_file.to_owned(),
                }
            })?;

            let expiry = create_staging_file()?;
            let mut expiry_file: &File = expiry.as_file();

            let result = if let Ok(expires_header) = headers.decode::<Expires>() {
                write!(expiry_file, "{}", expires_header)
            } else {
                let expiry_date = SystemTime::now() + Duration::from_secs(max_age(&headers).into());

                write!(expiry_file, "{}", HttpDate::from(expiry_date))
            };

            result.with_context(|| ErrorKind::WriteNodeIndexExpiryError {
                file: expiry.path().to_path_buf(),
            })?;

            let index_expiry_file = volta_home()?.node_index_expiry_file();
            ensure_containing_dir_exists(&index_expiry_file).with_context(|| {
                ErrorKind::ContainingDirError {
                    path: index_expiry_file.to_owned(),
                }
            })?;
            expiry.persist(&index_expiry_file).with_context(|| {
                ErrorKind::WriteNodeIndexExpiryError {
                    file: index_expiry_file.to_owned(),
                }
            })?;

            spinner.finish_and_clear();
            Ok(index)
        }
    }
}
