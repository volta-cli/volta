//! Provides types for installing packages to the user toolchain.

use inventory::RegistryFetchError;
use semver::Version;
use style::progress_spinner;
use version::VersionSpec;
use config::Config;
use distro::Fetched;
use distro::DistroVersion;

use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use notion_fail::FailExt;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;
use serde_json;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_package_registry_root() -> String {
            mockito::SERVER_URL.to_string()
        }
    } else {
        fn public_package_registry_root() -> String {
            "https://registry.npmjs.org".to_string()
        }
    }
}

/// Thrown when there is no Node version matching a requested semver specifier.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "No version of '{}' found for {}", name, matching)]
#[notion_fail(code = "NoVersionMatch")]
struct NoPackageFoundError {
    name: String,
    matching: VersionSpec,
}

/// A provisioned Package distribution.
pub struct PackageDistro {
    // TODO: I may need this later...
    // archive: Box<Archive>,
    name: String,
    version: Version,
}

impl PackageDistro {
    // TODO: will this need config?
    pub fn fetch(&self, config: &Config) -> Fallible<Fetched<DistroVersion>> {
        // TODO
        Ok(Fetched::Now(DistroVersion::Package(self.name.to_string(), self.version.clone())))
    }
}

/// Information about a package.
pub struct NpmPackage;

// TODO: description
pub struct PackageVersions {
    latest: Version,
    entries: Vec<Version>,
}

impl NpmPackage {

    // TODO: should this be &self method?
    pub fn resolve_public(name: &String, matching: &VersionSpec) -> Fallible<PackageDistro> {
        let versions: PackageVersions = resolve_package_metadata(name)?.into_versions()?;

        // TODO: this matching should be a method of PackageVersions?
        let version_opt = match *matching {
            VersionSpec::Latest => Some(versions.latest),
            VersionSpec::Semver(ref matching) => {
                match_package_version(versions, |ref v| matching.matches(v))?
            }
            VersionSpec::Exact(ref exact) => Some(exact.clone()),
        };

        if let Some(version) = version_opt {
            println!("matched {} version {}!", name, version);
            Ok(PackageDistro { name: name.to_string(), version: version })
        } else {
            throw!(NoPackageFoundError {
                name: name.to_string(),
                matching: matching.clone()
            })
        }
    }
}

// TODO
fn match_package_version(mut versions: PackageVersions, predicate: impl Fn(&Version) -> bool) -> Fallible<Option<Version>> {
    // sort versions, largest to smallest
    versions.entries.sort_by(|a, b| a.cmp(b).reverse());
    let mut entries = versions.entries.into_iter();
    Ok(entries.find(predicate))
}


// TODO: this has side effects, so needs acceptance & smoke tests
fn resolve_package_metadata(name: &String) -> Fallible<serial::PackageMetadata> {
    let package_info_uri = format!("{}/{}", public_package_registry_root(), name);
    let spinner = progress_spinner(&format!(
            "Fetching package metadata: {}",
            package_info_uri
            ));
    let mut response: reqwest::Response =
        reqwest::get(package_info_uri.as_str())
        .with_context(RegistryFetchError::from_error)?;
    let response_text: String = response.text().unknown()?;

    // TODO: caching for this? see inventory::resolve_node_versions() for an example of that

    let metadata: serial::PackageMetadata = serde_json::de::from_str(&response_text).unknown()?;

    spinner.finish_and_clear();
    Ok(metadata)
}

