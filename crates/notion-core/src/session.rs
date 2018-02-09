use config::{self, Config, NodeConfig, LazyConfig};
use plugin::{self, ResolveResponse};
use catalog::{Catalog, LazyCatalog};
use project::Project;
use failure;
use installer::node::Installer;
use serial;

use semver::{Version, VersionReq};
use reqwest;

use std::string::ToString;
use std::collections::{HashSet, BTreeMap};
use std::cmp::{Ord, PartialOrd, Ordering};

const PUBLIC_NODE_VERSION_INDEX: &'static str = "https://nodejs.org/dist/index.json";

pub struct Session {
    config: LazyConfig,
    catalog: LazyCatalog,
    project: Option<Project>
}

impl Session {

    pub fn new() -> Result<Session, failure::Error> {
        Ok(Session {
            config: LazyConfig::new(),
            catalog: LazyCatalog::new(),
            project: Project::for_current_dir()?
        })
    }

    pub fn catalog(&self) -> Result<&Catalog, failure::Error> {
        self.catalog.get()
    }

    pub fn catalog_mut(&mut self) -> Result<&mut Catalog, failure::Error> {
        self.catalog.get_mut()
    }

    pub fn config(&self) -> Result<&Config, failure::Error> {
        self.config.get()
    }

    // FIXME: should return Version once we kill lockfile
    pub fn node_version(&self) -> Result<Option<String>, failure::Error> {
        if let Some(ref project) = self.project {
            return Ok(Some(project.lockfile()?.node.version.clone()));
        }

        Ok(self.catalog()?.node.current.clone().map(|v| v.to_string()))
    }

    pub fn node(&mut self) -> Result<Option<Version>, failure::Error> {
        let req = if let Some(ref project) = self.project {
            Some(project.manifest().node_req())
        } else {
            None
        };

        if let Some(req) = req {
            let available = self.catalog()?.node.resolve_local(&req);

            return if available.is_some() {
                Ok(available)
            } else {
                let installer = self.resolve_remote_node(&req)?;
                let version = installer.install()?;
                self.catalog_mut()?.node.versions.insert(version.clone());
                self.catalog()?.save()?;
                Ok(Some(version))
            }
        }

        Ok(self.catalog()?.node.current.clone())
    }

    fn resolve_remote_node(&self, req: &VersionReq) -> Result<Installer, failure::Error> {
        let config = self.config()?;

        match config.node {
            Some(NodeConfig { resolve: Some(ref plugin), .. }) => {
                plugin.resolve(req)
            }
            _ => {
                self.resolve_public_node(req)
            }
        }
    }

    fn resolve_public_node(&self, req: &VersionReq) -> Result<Installer, failure::Error> {
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

}

pub struct Index {
    pub entries: BTreeMap<Version, VersionData>
}

pub struct VersionData {
    pub files: HashSet<String>
}
