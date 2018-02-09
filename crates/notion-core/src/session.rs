use config::{Config, LazyConfig};
use catalog::{Catalog, LazyCatalog};
use project::Project;
use failure;

use semver::Version;

use std::string::ToString;
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
        if let Some(ref project) = self.project {
            let req = project.manifest().node_req();
            let catalog = self.catalog.get_mut()?;
            let config = self.config.get()?;

            let available = catalog.node.resolve_local(&req);
            if available.is_some() {
                return Ok(available);
            }

            let version = catalog.install_req(&req, config)?;
            return Ok(Some(version));
        }

        Ok(self.catalog()?.node.current.clone())
    }

}

pub struct Index {
    pub entries: BTreeMap<Version, VersionData>
}

pub struct VersionData {
    pub files: HashSet<String>
}
