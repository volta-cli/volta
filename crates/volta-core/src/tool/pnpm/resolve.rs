use log::debug;
use node_semver::{Range, Version};

use crate::error::{ErrorKind, Fallible};
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::tool::registry::{fetch_npm_registry, public_registry_index, PackageIndex};
use crate::tool::{PackageDetails, Pnpm};
use crate::version::{VersionSpec, VersionTag};

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.pnpm();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None | VersionSpec::Tag(VersionTag::Latest) => resolve_tag("latest", hooks),
        VersionSpec::Tag(tag) => resolve_tag(&tag.to_string(), hooks),
    }
}

fn resolve_tag(tag: &str, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<Version> {
    let (url, mut index) = fetch_pnpm_index(hooks)?;

    match index.tags.remove(tag) {
        Some(version) => {
            debug!("Found pnpm@{} matching tag '{}' from {}", version, tag, url);
            Ok(version)
        }
        None => Err(ErrorKind::PnpmVersionNotFound {
            matching: tag.into(),
        }
        .into()),
    }
}

fn resolve_semver(matching: Range, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<Version> {
    let (url, index) = fetch_pnpm_index(hooks)?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.satisfies(version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found pnpm@{} matching requirement '{}' from {}",
                details.version, matching, url
            );
            Ok(details.version)
        }
        None => Err(ErrorKind::PnpmVersionNotFound {
            matching: matching.to_string(),
        }
        .into()),
    }
}

/// Fetch the index of available pnpm versions from the npm registry
fn fetch_pnpm_index(hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<(String, PackageIndex)> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using pnpm.index hook to determine pnpm index URL");
            hook.resolve("pnpm")?
        }
        _ => public_registry_index("pnpm"),
    };

    fetch_npm_registry(url, "pnpm")
}
