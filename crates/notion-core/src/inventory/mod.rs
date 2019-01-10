//! Provides types for working with Notion's _inventory_, the local repository
//! of available tool versions.

use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::str::FromStr;
use std::string::ToString;
use std::time::{Duration, SystemTime};

use lazycell::LazyCell;
use reqwest;
use reqwest::header::{CacheControl, CacheDirective, Expires, HttpDate};
use serde_json;
use tempfile::NamedTempFile;

use config::{Config, ToolConfig};
use distro::node::NodeDistro;
use distro::yarn::YarnDistro;
use distro::{Distro, Fetched};
use distro::DistroVersion;
use fs::{ensure_containing_dir_exists, read_file_opt};
use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};
use path;
use semver::{Version, VersionReq};
use style::progress_spinner;
use tool::ToolSpec;
use version::VersionSpec;

pub(crate) mod serial;

#[cfg(feature = "mock-network")]
use mockito;

// ISSUE (#86): Move public repository URLs to config file
cfg_if! {
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

/// The inventory of locally available tool versions.
pub struct Inventory {
    pub node: NodeCollection,
    pub yarn: YarnCollection,
}

impl Inventory {
    /// Returns the current inventory.
    fn current() -> Fallible<Inventory> {
        Ok(Inventory {
            node: NodeCollection::load()?,
            yarn: YarnCollection::load()?,
        })
    }

    /// Fetches a Tool version matching the specified semantic versioning requirements.
    pub fn fetch(&mut self, toolspec: &ToolSpec, config: &Config) -> Fallible<Fetched<DistroVersion>> {
        match toolspec {
            ToolSpec::Node(version) => self.node.fetch(&version, config),
            ToolSpec::Yarn(version) => self.yarn.fetch(&version, config),
            ToolSpec::Npm(_) => unimplemented!("TODO"),
            ToolSpec::Npx(_) => unimplemented!("TODO"),
            ToolSpec::Package(_, _) => unimplemented!("TODO"),
        }
    }
}

/// Thrown when there is no Node version matching a requested semver specifier.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "No Node version found for {}", matching)]
#[notion_fail(code = "NoVersionMatch")]
struct NoNodeVersionFoundError {
    matching: VersionSpec,
}

/// Thrown when there is no Yarn version matching a requested semver specifier.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "No Yarn version found for {}", matching)]
#[notion_fail(code = "NoVersionMatch")]
struct NoYarnVersionFoundError {
    matching: VersionReq,
}

impl<D: Distro> Collection<D> {
    /// Tests whether this Collection contains the specified Tool version.
    pub fn contains(&self, version: &Version) -> bool {
        self.versions.contains(version)
    }
}

pub trait FetchResolve<D: Distro> {
    /// Fetch a Distro version matching the specified semantic versioning requirements.
    fn fetch(&mut self, matching: &VersionSpec, config: &Config) -> Fallible<Fetched<DistroVersion>>;

    /// Resolves the specified semantic versioning requirements from a remote distributor.
    fn resolve_remote(
        &self,
        matching: &VersionSpec,
        config: Option<&ToolConfig<D>>,
    ) -> Fallible<D> {
        match config {
            Some(ToolConfig {
                resolve: Some(ref plugin),
                ..
            }) => plugin.resolve(matching),
            _ => self.resolve_public(matching),
        }
    }

    /// Resolves the specified semantic versioning requirements from the public distributor (e.g. `https://nodejs.org`).
    fn resolve_public(&self, matching: &VersionSpec) -> Fallible<D>;
}

/// Thrown when the public registry for Node or Yarn could not be downloaded.
#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Could not fetch public registry\n{}", error)]
#[notion_fail(code = "NetworkError")]
pub(crate) struct RegistryFetchError {
    error: String,
}

impl RegistryFetchError {
    pub(crate) fn from_error(error: &reqwest::Error) -> RegistryFetchError {
        RegistryFetchError {
            error: error.to_string(),
        }
    }
}

fn match_node_version(predicate: impl Fn(&NodeEntry) -> bool) -> Fallible<Option<Version>> {
    let index: NodeIndex = resolve_node_versions()?.into_index()?;
    let mut entries = index.entries.into_iter();
    Ok(entries.find(predicate).map(|NodeEntry { version, .. }| version))
}

impl FetchResolve<NodeDistro> for NodeCollection {
    fn fetch(&mut self, matching: &VersionSpec, config: &Config) -> Fallible<Fetched<DistroVersion>> {
        let distro = self.resolve_remote(matching, config.node.as_ref())?;
        let fetched = distro.fetch(&self).unknown()?;

        if let &Fetched::Now(DistroVersion::Node(ref version, .. )) = &fetched {
            self.versions.insert(version.clone());
        }

        Ok(fetched)
    }

    fn resolve_public(&self, matching: &VersionSpec) -> Fallible<NodeDistro> {
        let version_opt = match *matching {
            VersionSpec::Latest => {
                // NOTE: This assumes the registry always produces a list in sorted order
                //       from newest to oldest. This should be specified as a requirement
                //       when we document the plugin API.
                match_node_version(|_| true)?
            }
            VersionSpec::Semver(ref matching) => {
                // ISSUE #34: also make sure this OS is available for this version
                match_node_version(|&NodeEntry { version: ref v, .. }| matching.matches(v))?
            }
            VersionSpec::Exact(ref exact) => Some(exact.clone())
        };

        if let Some(version) = version_opt {
            NodeDistro::public(version)
        } else {
            throw!(NoNodeVersionFoundError {
                matching: matching.clone()
            })
        }
    }
}

impl FetchResolve<YarnDistro> for YarnCollection {
    /// Fetches a Yarn version matching the specified semantic versioning requirements.
    fn fetch(&mut self, matching: &VersionSpec, config: &Config) -> Fallible<Fetched<DistroVersion>> {
        let distro = self.resolve_remote(&matching, config.yarn.as_ref())?;
        let fetched = distro.fetch(&self).unknown()?;

        if let &Fetched::Now(DistroVersion::Yarn(ref version)) = &fetched {
            self.versions.insert(version.clone());
        }

        Ok(fetched)
    }

    /// Resolves the specified semantic versioning requirements from the public distributor.
    fn resolve_public(&self, matching: &VersionSpec) -> Fallible<YarnDistro> {
        let version = match *matching {
            VersionSpec::Latest => {
                let mut response: reqwest::Response =
                    reqwest::get(public_yarn_latest_version().as_str())
                        .with_context(RegistryFetchError::from_error)?;
                Version::parse(&response.text().unknown()?).unknown()?
            }
            VersionSpec::Semver(ref matching) => {
                let spinner = progress_spinner(&format!(
                    "Fetching public registry: {}",
                    public_yarn_version_index()
                ));
                let releases: serial::YarnIndex = reqwest::get(public_yarn_version_index().as_str())
                    .with_context(RegistryFetchError::from_error)?
                    .json()
                    .unknown()?;
                let releases = releases.into_index()?.entries;
                spinner.finish_and_clear();
                let version = releases.into_iter().rev().find(|v| {
                    matching.matches(v)
                });

                if let Some(version) = version {
                    version
                } else {
                    throw!(NoYarnVersionFoundError {
                        matching: matching.clone(),
                    });
                }
            },
            VersionSpec::Exact(ref exact) => exact.clone()
        };
        YarnDistro::public(version)
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
    pub files: NodeDistroFiles
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
    let expiry: Option<String> = read_file_opt(&path::node_index_expiry_file()?).unknown()?;

    if let Some(string) = expiry {
        let expiry_date: HttpDate = HttpDate::from_str(&string).unknown()?;
        let current_date: HttpDate = HttpDate::from(SystemTime::now());

        if current_date < expiry_date {
            let cached: Option<String> = read_file_opt(&path::node_index_file()?).unknown()?;

            if let Some(string) = cached {
                return Ok(serde_json::de::from_str(&string).unknown()?);
            }
        }
    }

    Ok(None)
}

/// Get the cache max-age of an HTTP reponse.
fn max_age(response: &reqwest::Response) -> u32 {
    if let Some(cache_control_header) = response.headers().get::<CacheControl>() {
        for cache_directive in cache_control_header.iter() {
            if let CacheDirective::MaxAge(max_age) = cache_directive {
                return *max_age;
            }
        }
    }

    // Default to four hours.
    4 * 60 * 60
}

fn resolve_node_versions() -> Fallible<serial::NodeIndex> {
    match read_cached_opt()? {
        Some(serial) => Ok(serial),
        None => {
            let spinner = progress_spinner(&format!(
                "Fetching public registry: {}",
                public_node_version_index()
            ));
            let mut response: reqwest::Response = reqwest::get(
                public_node_version_index().as_str(),
            ).with_context(RegistryFetchError::from_error)?;
            let response_text: String = response.text().unknown()?;
            let cached: NamedTempFile = NamedTempFile::new().unknown()?;

            // Block to borrow cached for cached_file.
            {
                let mut cached_file: &File = cached.as_file();
                cached_file.write(response_text.as_bytes()).unknown()?;
            }

            let index_cache_file = path::node_index_file()?;
            ensure_containing_dir_exists(&index_cache_file)?;
            cached.persist(index_cache_file).unknown()?;

            let expiry: NamedTempFile = NamedTempFile::new().unknown()?;

            // Block to borrow expiry for expiry_file.
            {
                let mut expiry_file: &File = expiry.as_file();

                if let Some(expires_header) = response.headers().get::<Expires>() {
                    write!(expiry_file, "{}", expires_header).unknown()?;
                } else {
                    let expiry_date =
                        SystemTime::now() + Duration::from_secs(max_age(&response).into());

                    write!(expiry_file, "{}", HttpDate::from(expiry_date)).unknown()?;
                }
            }

            let index_expiry_file = path::node_index_expiry_file()?;
            ensure_containing_dir_exists(&index_expiry_file)?;
            expiry.persist(index_expiry_file).unknown()?;

            let serial: serial::NodeIndex = serde_json::de::from_str(&response_text).unknown()?;

            spinner.finish_and_clear();
            Ok(serial)
        }
    }
}
