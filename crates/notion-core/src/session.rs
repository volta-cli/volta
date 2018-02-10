use config::{Config, LazyConfig};
use catalog::{Catalog, LazyCatalog};
use project::Project;
use failure;

use semver::{Version, VersionReq};

use std::collections::{HashSet, BTreeMap};

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

    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
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

    pub fn node(&mut self) -> Result<Option<Version>, failure::Error> {
        if let Some(ref project) = self.project {
            let req = project.manifest().node_req();
            let catalog = self.catalog.get_mut()?;
            let available = catalog.node.resolve_local(&req);

            if available.is_some() {
                return Ok(available);
            }

            let config = self.config.get()?;
            let version = catalog.install_req(&req, config)?;

            return Ok(Some(version));
        }

        Ok(self.catalog()?.node.current.clone())
    }

    pub fn install_node(&mut self, req: &VersionReq) -> Result<Version, failure::Error> {
        let catalog = self.catalog.get_mut()?;
        let config = self.config.get()?;
        catalog.install_req(req, config)
    }

    pub fn set_node_version(&mut self, req: &VersionReq) -> Result<(), failure::Error> {
        let catalog = self.catalog.get_mut()?;
        let config = self.config.get()?;
        catalog.set_version(req, config)
    }
}

pub struct Index {
    pub entries: BTreeMap<Version, VersionData>
}

pub struct VersionData {
    pub files: HashSet<String>
}
