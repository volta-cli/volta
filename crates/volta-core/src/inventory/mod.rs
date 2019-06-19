//! Provides types for working with Volta's _inventory_, the local repository
//! of available tool versions.

use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::process::Command;
use std::str::FromStr;
use std::string::ToString;
use std::time::{Duration, SystemTime};

use headers_011::Headers011;
use lazycell::LazyCell;
use log::debug;
use reqwest;
use reqwest::hyper_011::header::{CacheControl, CacheDirective, Expires, HttpDate};
use semver::{Version, VersionReq};
use serde_json;
use tempfile::NamedTempFile;
use volta_fail::{throw, Fallible, ResultExt};

use crate::command::create_command;
use crate::distro::node::{NodeDistro, NodeVersion};
use crate::distro::package::{PackageDistro, PackageEntry, PackageIndex, PackageVersion};
use crate::distro::yarn::YarnDistro;
use crate::distro::{Distro, Fetched};
use crate::error::ErrorDetails;
use crate::fs::{ensure_containing_dir_exists, read_file_opt};
use crate::hook::ToolHooks;
use crate::path;
use crate::style::progress_spinner;
use crate::style::tool_version;
use crate::version::VersionSpec;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;

// ISSUE (#86): Move public repository URLs to config file
cfg_if::cfg_if! {
    if #[cfg(feature = "mock-network")] {
        fn public_node_version_index() -> String {
            format!("{}/node-dist/index.json", mockito::SERVER_URL)
        }
        fn public_yarn_version_index() -> String {
            format!("{}/yarn-releases/index.json", mockito::SERVER_URL)
        }
        fn public_yarn_latest_version() -> String {
            format!("{}/yarn-latest", mockito::SERVER_URL)
        }
        // TODO: remove this
        // TODO: and what happens in the tests?
        // fn public_package_registry_root() -> String {
        //     format!("{}/registry", mockito::SERVER_URL)
        // }
    } else {
        /// Returns the URL of the index of available Node versions on the public Node server.
        fn public_node_version_index() -> String {
            "https://nodejs.org/dist/index.json".to_string()
        }
        /// Return the URL of the index of available Yarn versions on the public git repository.
        fn public_yarn_version_index() -> String {
            "https://api.github.com/repos/yarnpkg/yarn/releases".to_string()
        }
        /// URL of the latest Yarn version on the public yarnpkg.com
        fn public_yarn_latest_version() -> String {
            "https://yarnpkg.com/latest-version".to_string()
        }
        // TODO: remove this
        // /// URL of the Npm registry containing an index of availble public packages.
        // fn public_package_registry_root() -> String {
        //     "https://registry.npmjs.org".to_string()
        // }
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

fn match_node_version(
    url: &str,
    predicate: impl Fn(&NodeEntry) -> bool,
) -> Fallible<Option<Version>> {
    let index = resolve_node_versions(url)?.into_index()?;
    let mut entries = index.entries.into_iter();
    Ok(entries
        .find(predicate)
        .map(|NodeEntry { version, .. }| version))
}

impl FetchResolve<NodeDistro> for NodeCollection {
    type FetchedVersion = NodeVersion;

    fn fetch(
        &mut self,
        name: &str, // not used here, we already know this is "node"
        matching: &VersionSpec,
        hooks: Option<&ToolHooks<NodeDistro>>,
    ) -> Fallible<Fetched<NodeVersion>> {
        let distro = self.resolve(name, matching, hooks)?;
        let fetched = distro.fetch(&self)?;

        if let &Fetched::Now(NodeVersion { ref runtime, .. }) = &fetched {
            self.versions.insert(runtime.clone());
        }

        Ok(fetched)
    }

    fn resolve_latest(
        &self,
        _name: &str,
        hooks: Option<&ToolHooks<NodeDistro>>,
    ) -> Fallible<Version> {
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

        if let Some(version) = version_opt {
            debug!("Found latest node version ({}) from {}", version, url);
            Ok(version)
        } else {
            throw!(ErrorDetails::NodeVersionNotFound {
                matching: "latest".to_string()
            })
        }
    }

    fn resolve_semver(
        &self,
        _name: &str,
        matching: &VersionReq,
        hooks: Option<&ToolHooks<NodeDistro>>,
    ) -> Fallible<Version> {
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

        if let Some(version) = version_opt {
            debug!(
                "Found node@{} matching requirement '{}' from {}",
                version, matching, url
            );
            Ok(version)
        } else {
            throw!(ErrorDetails::NodeVersionNotFound {
                matching: matching.to_string()
            })
        }
    }

    fn resolve_lts(&self, _name: &str, hooks: Option<&ToolHooks<NodeDistro>>) -> Fallible<Version> {
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

        if let Some(version) = version_opt {
            debug!("Found newest LTS node version ({}) from {}", version, url);
            Ok(version)
        } else {
            throw!(ErrorDetails::NodeVersionNotFound {
                matching: "lts".to_string()
            })
        }
    }

    fn resolve_exact(
        &self,
        _name: &str,
        version: Version,
        _hooks: Option<&ToolHooks<NodeDistro>>,
    ) -> Fallible<Version> {
        Ok(version)
    }
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

// TODO: this should return a PackageIndex? that would be easier overall...
// use `npm view` to get the info for the package
// fn npm_view_query(name: &str, version: &str) -> Fallible<Vec<serial::NpmViewData>> {
fn npm_view_query(name: &str, version: &str) -> Fallible<PackageIndex> {
    let mut command = npm_view_command_for(name, version);
    debug!("Running command: `{:?}`", command);

    let spinner = progress_spinner(&format!(
        "Querying metadata for {}",
        tool_version(name, version)
    ));
    // TODO: different error for this - PackageMetadataFetchError?
    // TODO: don't have the verbose stuff in here, and don't write the verbose stuff to stdout????
    let output = command
        .output()
        .with_context(|_| ErrorDetails::PackageInstallFailed)?;
    spinner.finish_and_clear();

    debug!(
        "[package metadata stderr]\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // debug!(
    //     "[package metadata stdout]\n{}",
    //     String::from_utf8_lossy(&output.stdout)
    // );

    // TODO: different error for this - PackageMetadataFetchError?
    if !output.status.success() {
        throw!(ErrorDetails::PackageInstallFailed);
    }

    let response_json = String::from_utf8_lossy(&output.stdout);

    debug!("[first bit of response json]\n{:?}", &response_json[..80]);

    // TODO: instead of from_str here, sometimes this will return an array (semver case)
    // so, check if the first char is '[' ?!
    if response_json.chars().next() == Some('[') {
        // deserialize to array
        let metadatas = serde_json::de::from_str::<Vec<serial::NpmViewData>>(&response_json)
            .with_context(|_| {
                // TODO: this error needs to change
                ErrorDetails::ParsePackageMetadataError {
                    from_url: "TODO".to_string(),
                }
            })?;
        debug!("[parsed package metadata (from array)]\n{:?}", metadatas);

        // TODO: does this always work?
        let latest = metadatas[0].dist_tags.latest.clone();
        Ok(PackageIndex {
            latest,
            entries: metadatas.iter().map(|e| e.clone().into_index()).collect(),
        })
    } else {
        let metadata = serde_json::de::from_str::<serial::NpmViewData>(&response_json)
            .with_context(|_| {
                // TODO: this error needs to change
                ErrorDetails::ParsePackageMetadataError {
                    from_url: "TODO".to_string(),
                }
            })?;
        debug!("[parsed package metadata (single)]\n{:?}", metadata);

        let latest = metadata.dist_tags.latest.clone();
        Ok(PackageIndex {
            latest,
            entries: vec![metadata.into_index()],
        })
    }
}

// TODO: description
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
        let entry_opt = match hooks {
            Some(&ToolHooks {
                latest: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.latest hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                let package_index = resolve_package_metadata(name, &url)?.into_index();
                let latest = package_index.latest.clone();

                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    &latest == version
                })
            }
            _ => {
                // TODO: do the same matching thing here
                // (into_index will have to be changed)
                // OR....convert to the same PackageIndex format, to pull out the matching stuff?
                // let entries = npm_view_query(name, "latest")?
                //     .iter()
                //     .map(|e| e.into_index())
                //     .collect();
                // let package_index = PackageIndex {
                //     latest: Version::new(0, 0, 0),
                //     entries,
                // };

                let package_index = npm_view_query(name, "latest")?;

                // TODO:
                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    // &latest == version
                    true
                })
            }
        };

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
        let entry_opt = match hooks {
            Some(&ToolHooks {
                index: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.index hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                let package_index = resolve_package_metadata(name, &url)?.into_index();

                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    matching.matches(&version)
                })
            }
            _ => {
                // // TODO: do the same thing as above: get info for all versions, then iterate and
                // // find the one that matches
                // Some(npm_view_query(name, &matching.to_string())?.into_index())

                // TODO: do the same matching thing here
                // (into_index will have to be changed)
                // OR....convert to the same PackageIndex format, to pull out the matching stuff?
                // let entries = npm_view_query(name, &matching.to_string())?
                //     .iter()
                //     .map(|e| e.into_index())
                //     .collect();
                // // TODO:
                // let package_index = PackageIndex {
                //     latest: Version::new(0, 0, 0),
                //     entries,
                // };

                let package_index = npm_view_query(name, &matching.to_string())?;

                // TODO:
                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    // &latest == version
                    true
                })
            }
        };

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
        let entry_opt = match hooks {
            Some(&ToolHooks {
                index: Some(ref hook),
                ..
            }) => {
                debug!("Using packages.index hook to determine package metadata URL");
                let url = hook.resolve(&name)?;
                let package_index = resolve_package_metadata(name, &url)?.into_index();

                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    &exact_version == version
                })
            }
            _ => {
                // Some(npm_view_query(name, &exact_version.to_string())?.into_index()),
                // TODO: do the same matching thing here
                // (into_index will have to be changed)
                // OR....convert to the same PackageIndex format, to pull out the matching stuff?
                // let entries = npm_view_query(name, &exact_version.to_string())?
                //     .iter()
                //     .map(|e| e.into_index())
                //     .collect();
                // let package_index = PackageIndex {
                //     latest: Version::new(0, 0, 0),
                //     entries,
                // };

                let package_index = npm_view_query(name, &exact_version.to_string())?;

                // TODO:
                match_package_entry(package_index, |PackageEntry { version, .. }| {
                    // &latest == version
                    true
                })
            }
        };

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

/// The index of the public Node server.
pub struct NodeIndex {
    entries: Vec<NodeEntry>,
}

#[derive(Debug)]
pub struct NodeEntry {
    pub version: Version,
    pub npm: Version,
    pub files: NodeDistroFiles,
    pub lts: bool,
}

/// The public Yarn index.
pub struct YarnIndex {
    entries: BTreeSet<Version>,
}

/// The set of available files on the public Node server for a given Node version.
#[derive(Debug)]
pub struct NodeDistroFiles {
    pub files: HashSet<String>,
}

/// Reads a public index from the Node cache, if it exists and hasn't expired.
fn read_cached_opt() -> Fallible<Option<serial::NodeIndex>> {
    let expiry_file = path::node_index_expiry_file()?;
    let expiry = read_file_opt(&expiry_file)
        .with_context(|_| ErrorDetails::ReadNodeIndexExpiryError { file: expiry_file })?;

    if let Some(string) = expiry {
        let expiry_date = HttpDate::from_str(&string)
            .with_context(|_| ErrorDetails::ParseNodeIndexExpiryError)?;
        let current_date = HttpDate::from(SystemTime::now());

        if current_date < expiry_date {
            let index_file = path::node_index_file()?;
            let cached = read_file_opt(&index_file)
                .with_context(|_| ErrorDetails::ReadNodeIndexCacheError { file: index_file })?;

            if let Some(string) = cached {
                return serde_json::de::from_str(&string)
                    .with_context(|_| ErrorDetails::ParseNodeIndexCacheError);
            }
        }
    }

    Ok(None)
}

/// Get the cache max-age of an HTTP reponse.
fn max_age(response: &reqwest::Response) -> u32 {
    if let Some(cache_control_header) = response.headers().get_011::<CacheControl>() {
        for cache_directive in cache_control_header.iter() {
            if let CacheDirective::MaxAge(max_age) = cache_directive {
                return *max_age;
            }
        }
    }

    // Default to four hours.
    4 * 60 * 60
}

fn resolve_node_versions(url: &str) -> Fallible<serial::NodeIndex> {
    match read_cached_opt()? {
        Some(serial) => Ok(serial),
        None => {
            let spinner = progress_spinner(&format!("Fetching public registry: {}", url));

            let mut response: reqwest::Response =
                reqwest::get(url).with_context(registry_fetch_error("Node", url))?;
            let response_text = response
                .text()
                .with_context(registry_fetch_error("Node", url))?;
            let index: serial::NodeIndex =
                serde_json::de::from_str(&response_text).with_context(|_| {
                    ErrorDetails::ParseNodeIndexError {
                        from_url: url.to_string(),
                    }
                })?;

            let tmp_root = path::tmp_dir()?;
            // Helper to lazily determine temp dir string, without moving the file into the closures below
            let get_tmp_root = || tmp_root.to_owned();

            let cached = NamedTempFile::new_in(&tmp_root).with_context(|_| {
                ErrorDetails::CreateTempFileError {
                    in_dir: get_tmp_root(),
                }
            })?;

            // Block to borrow cached for cached_file.
            {
                let mut cached_file: &File = cached.as_file();
                cached_file
                    .write(response_text.as_bytes())
                    .with_context(|_| ErrorDetails::WriteNodeIndexCacheError {
                        file: cached.path().to_path_buf(),
                    })?;
            }

            let index_cache_file = path::node_index_file()?;
            ensure_containing_dir_exists(&index_cache_file)?;
            cached.persist(&index_cache_file).with_context(|_| {
                ErrorDetails::WriteNodeIndexCacheError {
                    file: index_cache_file,
                }
            })?;

            let expiry = NamedTempFile::new_in(&tmp_root).with_context(|_| {
                ErrorDetails::CreateTempFileError {
                    in_dir: get_tmp_root(),
                }
            })?;

            // Block to borrow expiry for expiry_file.
            {
                let mut expiry_file: &File = expiry.as_file();

                let result = if let Some(expires_header) = response.headers().get_011::<Expires>() {
                    write!(expiry_file, "{}", expires_header)
                } else {
                    let expiry_date =
                        SystemTime::now() + Duration::from_secs(max_age(&response).into());

                    write!(expiry_file, "{}", HttpDate::from(expiry_date))
                };

                result.with_context(|_| ErrorDetails::WriteNodeIndexExpiryError {
                    file: expiry.path().to_path_buf(),
                })?;
            }

            let index_expiry_file = path::node_index_expiry_file()?;
            ensure_containing_dir_exists(&index_expiry_file)?;
            expiry.persist(&index_expiry_file).with_context(|_| {
                ErrorDetails::WriteNodeIndexExpiryError {
                    file: index_expiry_file,
                }
            })?;

            spinner.finish_and_clear();
            Ok(index)
        }
    }
}
