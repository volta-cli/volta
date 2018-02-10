use std::collections::BTreeSet;
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

pub enum Installed {
    Already,
    Now
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

    // FIXME: belongs in NodeCatalog
    pub fn set_version(&mut self, req: &VersionReq, config: &Config) -> Result<(), failure::Error> {
        let version = self.install_req(req, config)?;
        self.node.current = Some(version);
        self.save()?;
        Ok(())
    }

    // FIXME: belongs in NodeCatalog
    pub fn install_req(&mut self, req: &VersionReq, config: &Config) -> Result<Version, failure::Error> {
        // FIXME: should get version from installer, not installer.install(), and don't install if it's already installed
        let installer = self.node.resolve_remote(&req, config)?;
        let version = installer.install()?;
        self.node.versions.insert(version.clone());
        self.save()?;
        Ok(version)
    }

    pub fn uninstall(&mut self, version: &Version) -> Result<(), failure::Error> {
        if self.node.versions.contains(version) {
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
            // FIXME: also make sure this OS is available for this version
            .skip_while(|&(ref k, _)| !req.matches(k))
            .next()
            .map(|(k, _)| k.clone());
        if let Some(version) = version {
            Installer::public(version)
        } else {
            // FIXME: throw an error there
            panic!("no version {}", req)
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
