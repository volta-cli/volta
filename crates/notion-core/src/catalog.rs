use std::collections::BTreeSet;
use std::fs::{File, remove_dir_all};
use std::io::{self, Write};
use std::str::FromStr;
use std::string::ToString;

use lazycell::LazyCell;
use readext::ReadExt;
use reqwest;
use toml;

use super::NoNodeVersionFoundError;
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

    pub fn set_node_version(&mut self, req: &VersionReq, config: &Config) -> Result<(), failure::Error> {
        let installed = self.install_node_req(req, config)?;
        let version = Some(installed.into_version());

        if self.node.current != version {
            self.node.current = version;
            self.save()?;
        }

        Ok(())
    }

    pub fn install_node_req(&mut self, req: &VersionReq, config: &Config) -> Result<Installed, failure::Error> {
        let installer = self.node.resolve_remote(&req, config)?;
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

impl NodeCatalog {

    pub fn contains(&self, version: &Version) -> bool {
        self.versions.contains(version)
    }

    fn resolve_remote(&self, req: &VersionReq, config: &Config) -> Result<Installer, failure::Error> {
        match config.node {
            Some(NodeConfig { resolve: Some(ref plugin), .. }) => {
                plugin.resolve(req)
            }
            _ => {
                self.resolve_public(req)
            }
        }
    }

    fn resolve_public(&self, req: &VersionReq) -> Result<Installer, failure::Error> {
        let serial: serial::index::Index = reqwest::get(PUBLIC_NODE_VERSION_INDEX)?.json()?;
        let index = serial.into_index()?;
        let version = index.entries.iter()
            .rev()
            // ISSUE #34: also make sure this OS is available for this version
            .skip_while(|&(ref k, _)| !req.matches(k))
            .next()
            .map(|(k, _)| k.clone());
        if let Some(version) = version {
            Installer::public(version)
        } else {
            Err(NoNodeVersionFoundError { req: req.clone() }.into())
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

impl FromStr for Catalog {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let serial: serial::catalog::Catalog = toml::from_str(src)?;
        Ok(serial.into_catalog()?)
    }
}
