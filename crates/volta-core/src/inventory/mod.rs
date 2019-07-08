//! Provides types for working with Volta's _inventory_, the local repository
//! of available tool versions.

use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::process::Command;
use std::string::ToString;

use lazycell::LazyCell;
use log::debug;
use reqwest;
use semver::{Version, VersionReq};
use serde_json;
use volta_fail::{throw, Fallible, ResultExt};

use crate::command::create_command;
use crate::distro::node::NodeDistro;
use crate::distro::package::{PackageDistro, PackageEntry, PackageIndex, PackageVersion};
use crate::distro::yarn::YarnDistro;
use crate::distro::{Distro, Fetched};
use crate::error::ErrorDetails;
use crate::hook::ToolHooks;
use crate::style::progress_spinner;
use crate::style::tool_version;
use crate::version::VersionSpec;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;

// ISSUE (#86): Move public repository URLs to config file
cfg_if::cfg_if! {
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

/// Lazily loaded inventory.
pub struct LazyInventory {
    inventory: LazyCell<Inventory>,
}

impl LazyInventory {
    /// Constructs a new `LazyInventory`.
    pub fn new() -> LazyInventory {
        LazyInventory {
            inventory: LazyCell::new(),
        }
    }

    /// Forces the loading of the inventory and returns an immutable reference to it.
    pub fn get(&self) -> Fallible<&Inventory> {
        self.inventory.try_borrow_with(|| Inventory::current())
    }

    /// Forces the loading of the inventory and returns a mutable reference to it.
    pub fn get_mut(&mut self) -> Fallible<&mut Inventory> {
        self.inventory.try_borrow_mut_with(|| Inventory::current())
    }
}

pub struct Collection<D: Distro> {
    // A sorted collection of the available versions in the inventory.
    pub versions: BTreeSet<Version>,

    pub phantom: PhantomData<D>,
}

pub type NodeCollection = Collection<NodeDistro>;
pub type YarnCollection = Collection<YarnDistro>;
pub type PackageCollection = Collection<PackageDistro>;

/// The inventory of locally available tool versions.
pub struct Inventory {
    pub node: NodeCollection,
    pub yarn: YarnCollection,
    pub packages: PackageCollection,
}

impl Inventory {
    /// Returns the current inventory.
    fn current() -> Fallible<Inventory> {
        Ok(Inventory {
            node: NodeCollection::load()?,
            yarn: YarnCollection::load()?,
            packages: PackageCollection::load()?,
        })
    }
}

impl<D: Distro> Collection<D> {
    /// Tests whether this Collection contains the specified Tool version.
    pub fn contains(&self, version: &Version) -> bool {
        self.versions.contains(version)
    }
}

pub trait FetchResolve<D: Distro> {
    type FetchedVersion;

    /// Fetch a Distro version matching the specified semantic versioning requirements.
    fn fetch(
        &mut self,
        name: &str, // unused by Node and Yarn, but package install needs this
        matching: &VersionSpec,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<Fetched<Self::FetchedVersion>>;

    /// Resolves the specified semantic versioning requirements into a distribution
    fn resolve(
        &self,
        name: &str,
        matching: &VersionSpec,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<D> {
        let version = match matching {
            VersionSpec::Latest => self.resolve_latest(&name, hooks)?,
            VersionSpec::Lts => self.resolve_lts(&name, hooks)?,
            VersionSpec::Semver(requirement) => self.resolve_semver(&name, requirement, hooks)?,
            VersionSpec::Exact(version) => self.resolve_exact(&name, version.to_owned(), hooks)?,
        };

        D::new(name, version, hooks)
    }

    /// Resolves the latest version for this tool, using either the `latest` hook or the public registry
    fn resolve_latest(
        &self,
        name: &str,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<D::ResolvedVersion>;

    /// Resolves a SemVer version for this tool, using either the `index` hook or the public registry
    fn resolve_semver(
        &self,
        name: &str,
        matching: &VersionReq,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<D::ResolvedVersion>;

    /// Resolves an LTS version for this tool
    fn resolve_lts(
        &self,
        name: &str,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<D::ResolvedVersion> {
        return self.resolve_latest(name, hooks);
    }

    /// Resolves an exact version of this tool
    fn resolve_exact(
        &self,
        name: &str,
        version: Version,
        hooks: Option<&ToolHooks<D>>,
    ) -> Fallible<D::ResolvedVersion>;
}

fn registry_fetch_error(
    tool: impl AsRef<str>,
    from_url: impl AsRef<str>,
) -> impl FnOnce(&reqwest::Error) -> ErrorDetails {
    let tool = tool.as_ref().to_string();
    let from_url = from_url.as_ref().to_string();
    |_| ErrorDetails::RegistryFetchError { tool, from_url }
}

impl FetchResolve<YarnDistro> for YarnCollection {
    type FetchedVersion = Version;

    /// Fetches a Yarn version matching the specified semantic versioning requirements.
    fn fetch(
        &mut self,
        name: &str, // not used here, we already know this is "yarn"
        matching: &VersionSpec,
        hooks: Option<&ToolHooks<YarnDistro>>,
    ) -> Fallible<Fetched<Self::FetchedVersion>> {
        let distro = self.resolve(name, &matching, hooks)?;
        let fetched = distro.fetch(&self)?;

        if let &Fetched::Now(ref version) = &fetched {
            self.versions.insert(version.clone());
        }

        Ok(fetched)
    }

    fn resolve_latest(
        &self,
        _name: &str,
        hooks: Option<&ToolHooks<YarnDistro>>,
    ) -> Fallible<Version> {
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

    fn resolve_semver(
        &self,
        _name: &str,
        matching: &VersionReq,
        hooks: Option<&ToolHooks<YarnDistro>>,
    ) -> Fallible<Version> {
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
        let releases: serial::YarnIndex = reqwest::get(&url)
            .and_then(|mut resp| resp.json())
            .with_context(registry_fetch_error("Yarn", &url))?;
        let releases = releases.into_index()?.entries;
        spinner.finish_and_clear();
        let version_opt = releases.into_iter().rev().find(|v| matching.matches(v));

        if let Some(version) = version_opt {
            debug!(
                "Found yarn@{} matching requirement '{}' from {}",
                version, matching, url
            );
            Ok(version)
        } else {
            throw!(ErrorDetails::YarnVersionNotFound {
                matching: matching.to_string()
            })
        }
    }

    fn resolve_exact(
        &self,
        _name: &str,
        version: Version,
        _hooks: Option<&ToolHooks<YarnDistro>>,
    ) -> Fallible<Version> {
        Ok(version)
    }
}

// use the input predicate to match a package in the index
fn match_package_entry(
    index: PackageIndex,
    predicate: impl Fn(&PackageEntry) -> bool,
) -> Option<PackageEntry> {
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
        let metadatas: Vec<serial::NpmViewData> = serde_json::de::from_str(&response_json)
            .with_context(|_| ErrorDetails::NpmViewMetadataParseError)?;
        debug!("[parsed package metadata (array)]\n{:?}", metadatas);

        // get latest version, making sure the array is not empty
        let latest = match metadatas.iter().next() {
            Some(m) => m.dist_tags.latest.clone(),
            None => throw!(ErrorDetails::PackageNotFound {
                package: name.to_string()
            }),
        };

        let mut entries: Vec<PackageEntry> = metadatas.into_iter().map(|e| e.into()).collect();
        // sort so that the versions are ordered highest-to-lowest
        entries.sort_by(|a, b| b.version.cmp(&a.version));

        debug!("[sorted entries]\n{:?}", entries);

        Ok(PackageIndex { latest, entries })
    } else {
        let metadata: serial::NpmViewData = serde_json::de::from_str(&response_json)
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
) -> Fallible<serial::PackageMetadata> {
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

    let metadata: serial::PackageMetadata =
        serde_json::de::from_str(&response_text).with_context(|_| {
            ErrorDetails::ParsePackageMetadataError {
                from_url: package_info_url.to_string(),
            }
        })?;

    spinner.finish_and_clear();
    Ok(metadata)
}

impl FetchResolve<PackageDistro> for PackageCollection {
    type FetchedVersion = PackageVersion;

    /// Fetches a package version matching the specified semantic versioning requirements.
    fn fetch(
        &mut self,
        name: &str,
        matching: &VersionSpec,
        hooks: Option<&ToolHooks<PackageDistro>>,
    ) -> Fallible<Fetched<Self::FetchedVersion>> {
        let distro = self.resolve(name, &matching, hooks)?;
        let fetched = distro.fetch(&self)?;

        if let &Fetched::Now(PackageVersion { ref version, .. }) = &fetched {
            self.versions.insert(version.clone());
        }

        Ok(fetched)
    }

    fn resolve_latest(
        &self,
        name: &str,
        hooks: Option<&ToolHooks<PackageDistro>>,
    ) -> Fallible<PackageEntry> {
        let package_index = match hooks {
            Some(&ToolHooks {
                latest: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.latest hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                resolve_package_metadata(name, &url)?.into_index()
            }
            _ => npm_view_query(name, "latest")?,
        };

        let latest = package_index.latest.clone();
        let entry_opt = match_package_entry(package_index, |PackageEntry { version, .. }| {
            &latest == version
        });

        if let Some(entry) = entry_opt {
            debug!(
                "Found {} latest version ({}) from {}",
                name, entry.version, entry.tarball
            );
            Ok(entry)
        } else {
            throw!(ErrorDetails::PackageVersionNotFound {
                name: name.to_string(),
                matching: String::from("latest"),
            })
        }
    }

    fn resolve_semver(
        &self,
        name: &str,
        matching: &VersionReq,
        hooks: Option<&ToolHooks<PackageDistro>>,
    ) -> Fallible<PackageEntry> {
        let package_index = match hooks {
            Some(&ToolHooks {
                index: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.index hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                resolve_package_metadata(name, &url)?.into_index()
            }
            _ => npm_view_query(name, &matching.to_string())?,
        };

        let entry_opt = match_package_entry(package_index, |PackageEntry { version, .. }| {
            matching.matches(&version)
        });

        if let Some(entry) = entry_opt {
            debug!(
                "Found {}@{} matching requirement '{}' from {}",
                name, entry.version, matching, entry.tarball
            );
            Ok(entry)
        } else {
            throw!(ErrorDetails::PackageVersionNotFound {
                name: name.to_string(),
                matching: matching.to_string(),
            })
        }
    }

    fn resolve_exact(
        &self,
        name: &str,
        exact_version: Version,
        hooks: Option<&ToolHooks<PackageDistro>>,
    ) -> Fallible<PackageEntry> {
        let package_index = match hooks {
            Some(&ToolHooks {
                index: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.index hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                resolve_package_metadata(name, &url)?.into_index()
            }
            _ => npm_view_query(name, &exact_version.to_string())?,
        };

        let entry_opt = match_package_entry(package_index, |PackageEntry { version, .. }| {
            &exact_version == version
        });

        if let Some(entry) = entry_opt {
            debug!("Found {}@{} from {}", name, entry.version, entry.tarball);
            Ok(entry)
        } else {
            throw!(ErrorDetails::PackageVersionNotFound {
                name: name.to_string(),
                matching: exact_version.to_string(),
            })
        }
    }
}

/// The public Yarn index.
pub struct YarnIndex {
    entries: BTreeSet<Version>,
}
