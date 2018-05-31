//! Provides types for working with Notion's local _catalog_, the local repository
//! of available tool versions.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, remove_dir_all, File};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::time::SystemTime;

use lazycell::LazyCell;
use readext::ReadExt;
use reqwest;
use reqwest::header::HttpDate;
use serde_json;
use tempfile::NamedTempFile;
use toml;

use path::{self, user_catalog_file};
use serial::touch;
use notion_fail::{FailExt, Fallible, NotionError, ResultExt};
use semver::{Version, VersionReq};
use installer::Installed;
use installer::node::Installer;
use serial;
use config::{Config, NodeConfig};
use style::progress_spinner;

/// URL of the index of available Node versions on the public Node server.
const PUBLIC_NODE_VERSION_INDEX: &'static str = "https://nodejs.org/dist/index.json";

/// Lazily loaded tool catalog.
pub struct LazyCatalog {
    catalog: LazyCell<Catalog>,
}

impl LazyCatalog {
    /// Constructs a new `LazyCatalog`.
    pub fn new() -> LazyCatalog {
        LazyCatalog {
            catalog: LazyCell::new(),
        }
    }

    /// Forces the loading of the catalog and returns an immutable reference to it.
    pub fn get(&self) -> Fallible<&Catalog> {
        self.catalog.try_borrow_with(|| Catalog::current())
    }

    /// Forces the loading of the catalog and returns a mutable reference to it.
    pub fn get_mut(&mut self) -> Fallible<&mut Catalog> {
        self.catalog.try_borrow_mut_with(|| Catalog::current())
    }
}

/// The catalog of tool versions available locally.
pub struct Catalog {
    pub node: NodeCatalog,
}

/// The catalog of Node versions available locally.
pub struct NodeCatalog {
    /// The currently activated Node version, if any.
    pub activated: Option<Version>,

    // A sorted collection of the available versions in the catalog.
    pub versions: BTreeSet<Version>,
}

impl Catalog {
    /// Returns the current tool catalog.
    fn current() -> Fallible<Catalog> {
        let path = user_catalog_file()?;
        let src = touch(&path)?.read_into_string().unknown()?;
        src.parse()
    }

    /// Returns a pretty-printed TOML representation of the contents of the catalog.
    pub fn to_string(&self) -> String {
        toml::to_string_pretty(&self.to_serial()).unwrap()
    }

    /// Saves the contents of the catalog to the user's catalog file.
    pub fn save(&self) -> Fallible<()> {
        let path = user_catalog_file()?;
        let mut file = File::create(&path).unknown()?;
        file.write_all(self.to_string().as_bytes()).unknown()?;
        Ok(())
    }

    /// Activates a Node version matching the specified semantic versioning requirements.
    pub fn activate_node(&mut self, matching: &VersionReq, config: &Config) -> Fallible<()> {
        let installed = self.install_node(matching, config)?;
        let version = Some(installed.into_version());

        if self.node.activated != version {
            self.node.activated = version;
            self.save()?;
        }

        Ok(())
    }

    /// Installs a Node version matching the specified semantic versioning requirements.
    pub fn install_node(&mut self, matching: &VersionReq, config: &Config) -> Fallible<Installed> {
        let installer = self.node.resolve_remote(&matching, config)?;
        let installed = installer.install(&self.node).unknown()?;

        if let &Installed::Now(ref version) = &installed {
            self.node.versions.insert(version.clone());
            self.save()?;
        }

        Ok(installed)
    }

    /// Uninstalls a specific Node version from the local catalog.
    pub fn uninstall_node(&mut self, version: &Version) -> Fallible<()> {
        if self.node.contains(version) {
            let home = path::node_version_dir(&version.to_string())?;

            if !home.is_dir() {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{} is not a directory", home.to_string_lossy()),
                )).unknown()?;
            }

            remove_dir_all(home).unknown()?;

            self.node.versions.remove(version);

            self.save()?;
        }

        Ok(())
    }
}

/// Thrown when there is no Node version matching a requested semver specifier.
#[derive(Fail, Debug)]
#[fail(display = "No Node version found for {}", matching)]
struct NoNodeVersionFoundError {
    matching: VersionReq,
}

/// Reads a file, if it exists.
fn maybe_read_file(path: &PathBuf) -> io::Result<Option<String>> {
    let result: io::Result<String> = fs::read_to_string(path);

    match result {
        Ok(string) => Ok(Some(string)),

        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(error)
            }
        },
    }
}

/// Reads a public index from the Node cache, if it exists and hasn't expired.
fn read_cached() -> Fallible<Option<serial::index::Index>> {
    let expiry: Option<String> = maybe_read_file(&path::node_index_expiry_file()?).unknown()?;

    if let Some(string) = expiry {
        let expiry_date: HttpDate = HttpDate::from_str(&string).unknown()?;
        let current_date: HttpDate = HttpDate::from(SystemTime::now());

        if current_date < expiry_date {
            let cached: Option<String> = maybe_read_file(&path::node_index_file()?).unknown()?;

            if let Some(string) = cached {
                return Ok(serde_json::de::from_str(&string).unknown()?);
            }
        }
    }

    Ok(None)
}

impl NodeCatalog {
    /// Tests whether this Node catalog contains the specified Node version.
    pub fn contains(&self, version: &Version) -> bool {
        self.versions.contains(version)
    }

    /// Resolves the specified semantic versioning requirements from a remote distributor.
    fn resolve_remote(&self, matching: &VersionReq, config: &Config) -> Fallible<Installer> {
        match config.node {
            Some(NodeConfig {
                resolve: Some(ref plugin),
                ..
            }) => plugin.resolve(matching),
            _ => self.resolve_public(matching),
        }
    }

    /// Resolves the specified semantic versioning requirements from the public distributor (`https://nodejs.org`).
    fn resolve_public(&self, matching: &VersionReq) -> Fallible<Installer> {
        let index: Index = match read_cached().unknown()? {
            Some(serial) => serial,
            None => {
                let spinner = progress_spinner(&format!(
                    "Fetching public registry: {}",
                    PUBLIC_NODE_VERSION_INDEX
                ));
                let mut response: reqwest::Response = reqwest::get(PUBLIC_NODE_VERSION_INDEX).unknown()?;
                let cached: NamedTempFile = NamedTempFile::new().unknown()?;

                response.copy_to(&mut cached.as_file()).unknown()?;
                cached.persist(path::node_index_file()?).unknown()?;

                let expiry: NamedTempFile = NamedTempFile::new().unknown()?;

                // Block to borrow expiry for expiry_file.
                {
                    let mut expiry_file = expiry.as_file();
                    let header: &reqwest::header::Raw = response.headers().get_raw("Expires").unwrap();

                    for line in header.iter() {
                        expiry_file.write(line).unknown()?;
                    }
                }

                expiry.persist(path::node_index_expiry_file()?).unknown()?;

                let serial: serial::index::Index = response.json().unknown()?;

                spinner.finish_and_clear();
                serial
            },
        }.into_index()?;

        let version = index.entries.iter()
            .rev()
            // ISSUE #34: also make sure this OS is available for this version
            .skip_while(|&(ref k, _)| !matching.matches(k))
            .next()
            .map(|(k, _)| k.clone());
        if let Some(version) = version {
            Installer::public(version)
        } else {
            throw!(
                NoNodeVersionFoundError {
                    matching: matching.clone(),
                }.unknown()
            );
        }
    }

    /// Resolves the specified semantic versioning requirements from the local catalog.
    pub fn resolve_local(&self, req: &VersionReq) -> Option<Version> {
        self.versions
            .iter()
            .rev()
            .skip_while(|v| !req.matches(&v))
            .next()
            .map(|v| v.clone())
    }
}

/// The index of the public Node server.
pub struct Index {
    pub entries: BTreeMap<Version, VersionData>,
}

/// The set of available files on the public Node server for a given Node version.
pub struct VersionData {
    pub files: HashSet<String>,
}

impl FromStr for Catalog {
    type Err = NotionError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::catalog::Catalog = toml::from_str(src).unknown()?;
        Ok(serial.into_catalog()?)
    }
}
