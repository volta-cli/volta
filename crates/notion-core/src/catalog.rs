//! Provides types for working with Notion's local _catalog_, the local repository
//! of available tool versions.

use std::collections::{HashSet, BTreeSet, BTreeMap};
use std::fs::{File, remove_dir_all};
use std::io::{self, Write};
use std::str::FromStr;
use std::string::ToString;

use lazycell::LazyCell;
use readext::ReadExt;
use reqwest;
use toml;

use path::{self, user_catalog_file};
use serial::touch;
use failure;
use semver::{Version, VersionReq};
use installer::Installed;
use installer::node::Installer;
use serial;
use config::{Config, NodeConfig};

const PUBLIC_NODE_VERSION_INDEX: &'static str = "https://nodejs.org/dist/index.json";

pub struct LazyCatalog {
    catalog: LazyCell<Catalog>
}

impl LazyCatalog {
    pub fn new() -> LazyCatalog {
        LazyCatalog {
            catalog: LazyCell::new()
        }
    }

    pub fn get(&self) -> Result<&Catalog, failure::Error> {
        self.catalog.try_borrow_with(|| Catalog::current())
    }

    pub fn get_mut(&mut self) -> Result<&mut Catalog, failure::Error> {
        self.catalog.try_borrow_mut_with(|| Catalog::current())
    }
}

pub struct Catalog {
    pub node: NodeCatalog
}

pub struct NodeCatalog {
    pub current: Option<Version>,

    // A sorted collection of the available versions in the catalog.
    pub versions: BTreeSet<Version>
}

impl Catalog {

    pub fn current() -> Result<Catalog, failure::Error> {
        let path = user_catalog_file()?;
        let src = touch(&path)?.read_into_string()?;
        src.parse()
    }

    pub fn to_string(&self) -> String {
        toml::to_string_pretty(&self.to_serial()).unwrap()
    }

    pub fn save(&self) -> Result<(), failure::Error> {
        let path = user_catalog_file()?;
        let mut file = File::create(&path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    pub fn activate_node(&mut self, matching: &VersionReq, config: &Config) -> Result<(), failure::Error> {
        let installed = self.install_node(matching, config)?;
        let version = Some(installed.into_version());

        if self.node.current != version {
            self.node.current = version;
            self.save()?;
        }

        Ok(())
    }

    pub fn install_node(&mut self, matching: &VersionReq, config: &Config) -> Result<Installed, failure::Error> {
        let installer = self.node.resolve_remote(&matching, config)?;
        let installed = installer.install(&self.node)?;

        if let &Installed::Now(ref version) = &installed {
            self.node.versions.insert(version.clone());
            self.save()?;
        }

        Ok(installed)
    }

    pub fn uninstall_node(&mut self, version: &Version) -> Result<(), failure::Error> {
        if self.node.contains(version) {
            let home = path::node_version_dir(&version.to_string())?;

            if !home.is_dir() {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{} is not a directory", home.to_string_lossy())))?;
            }

            remove_dir_all(home)?;

            self.node.versions.remove(version);

            self.save()?;
        }

        Ok(())
    }

}

#[derive(Fail, Debug)]
#[fail(display = "No Node version found for {}", matching)]
struct NoNodeVersionFoundError {
    matching: VersionReq
}

impl NodeCatalog {

    pub fn contains(&self, version: &Version) -> bool {
        self.versions.contains(version)
    }

    fn resolve_remote(&self, matching: &VersionReq, config: &Config) -> Result<Installer, failure::Error> {
        match config.node {
            Some(NodeConfig { resolve: Some(ref plugin), .. }) => {
                plugin.resolve(matching)
            }
            _ => {
                self.resolve_public(matching)
            }
        }
    }

    fn resolve_public(&self, matching: &VersionReq) -> Result<Installer, failure::Error> {
        let serial: serial::index::Index = reqwest::get(PUBLIC_NODE_VERSION_INDEX)?.json()?;
        let index = serial.into_index()?;
        let version = index.entries.iter()
            .rev()
            // ISSUE #34: also make sure this OS is available for this version
            .skip_while(|&(ref k, _)| !matching.matches(k))
            .next()
            .map(|(k, _)| k.clone());
        if let Some(version) = version {
            Installer::public(version)
        } else {
            Err(NoNodeVersionFoundError { matching: matching.clone() }.into())
        }
    }

    pub fn resolve_local(&self, req: &VersionReq) -> Option<Version> {
        self.versions
            .iter()
            .rev()
            .skip_while(|v| !req.matches(&v))
            .next()
            .map(|v| v.clone())
    }

}

pub struct Index {
    pub entries: BTreeMap<Version, VersionData>
}

pub struct VersionData {
    pub files: HashSet<String>
}

impl FromStr for Catalog {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::catalog::Catalog = toml::from_str(src)?;
        Ok(serial.into_catalog()?)
    }
}
