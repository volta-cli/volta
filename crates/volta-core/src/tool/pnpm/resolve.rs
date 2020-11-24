use super::super::registry::{
    public_registry_index, PackageDetails, PackageIndex, RawPackageMetadata,
    NPM_ABBREVIATED_ACCEPT_HEADER,
};
use super::super::registry_fetch_error;
use super::Pnpm;
use crate::error::{Context, ErrorKind, Fallible};
use crate::hook::ToolHooks;
use crate::session::Session;
use crate::style::progress_spinner;
use crate::version::{VersionSpec, VersionTag};
use attohttpc::header::ACCEPT;
use attohttpc::Response;
use log::debug;
use semver::{Version, VersionReq};

pub fn resolve(matching: VersionSpec, session: &mut Session) -> Fallible<Version> {
    let hooks = session.hooks()?.pnpm();
    match matching {
        VersionSpec::Semver(requirement) => resolve_semver(requirement, hooks),
        VersionSpec::Exact(version) => Ok(version),
        VersionSpec::None => resolve_tag(VersionTag::Latest, hooks),
        VersionSpec::Tag(tag) => resolve_tag(tag, hooks),
    }
}

fn resolve_tag(tag: VersionTag, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<Version> {
    let (url, mut index) = fetch_pnpm_index(hooks)?;
    let tag = tag.to_string();

    match index.tags.remove(&tag) {
        Some(version) => {
            debug!("Found pnpm@{} matching tag '{}' from {}", version, tag, url);
            Ok(version)
        }
        None => Err(ErrorKind::PnpmVersionNotFound { matching: tag }.into()),
    }
}

fn resolve_semver(matching: VersionReq, hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<Version> {
    let (url, index) = fetch_pnpm_index(hooks)?;

    let details_opt = index
        .entries
        .into_iter()
        .find(|PackageDetails { version, .. }| matching.matches(&version));

    match details_opt {
        Some(details) => {
            debug!(
                "Found npm@{} matching requirement '{}' from {}",
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

fn fetch_pnpm_index(hooks: Option<&ToolHooks<Pnpm>>) -> Fallible<(String, PackageIndex)> {
    let url = match hooks {
        Some(&ToolHooks {
            index: Some(ref hook),
            ..
        }) => {
            debug!("Using pnpm.index hook to determing pnpm index URL");
            hook.resolve("pnpm")?
        }
        _ => public_registry_index("pnpm"),
    };

    let spinner = progress_spinner(&format!("Fetching public registry: {}", url));
    let metadata: RawPackageMetadata = attohttpc::get(&url)
        .header(ACCEPT, NPM_ABBREVIATED_ACCEPT_HEADER)
        .send()
        .and_then(Response::error_for_status)
        .and_then(Response::json)
        .with_context(registry_fetch_error("pnpm", &url))?;

    spinner.finish_and_clear();
    Ok((url, metadata.into()))
}
