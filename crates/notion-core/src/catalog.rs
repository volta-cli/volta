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
use untoml::touch;
use provision;
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

    pub fn set_version(&mut self, version: Version) -> Result<(), failure::Error> {
        self.install(&version.to_string())?;
        self.node.current = Some(version);
        self.save()?;
        Ok(())
    }

    pub fn install_req(&mut self, req: &VersionReq, config: &Config) -> Result<Version, failure::Error> {
        let installer = self.node.resolve_remote(&req, config)?;
        let version = installer.install()?;
        self.node.versions.insert(version.clone());
        self.save()?;
        Ok(version)
    }

    // FIXME: this should take semver::Version instead
    pub fn install(&mut self, version: &str) -> Result<Installed, failure::Error> {
        // FIXME: this should be based on the data structure instead
        if path::node_version_dir(version)?.is_dir() {
            Ok(Installed::Already)
        } else {
            provision::by_version(version)?;
            // FIXME: update the data structure and self.save()
            Ok(Installed::Now)
        }
    }

    // FIXME: this should take semver::Version instead
    pub fn uninstall(&mut self, version: &str) -> Result<(), failure::Error> {
        let home = path::node_version_dir(version)?;

        // FIXME: this should be based on the data structure instead
        if !home.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a directory", home.to_string_lossy())))?;
        }

        remove_dir_all(home)?;

        // FIXME: update the data structure and self.save()

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
