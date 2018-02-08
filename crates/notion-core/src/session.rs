use config::{self, Config, NodeConfig};
use plugin::{self, ResolveResponse};
use catalog::Catalog;
use project::Project;
use failure;
use installer::node::Installer;

use lazycell::LazyCell;
use semver::{Version, VersionReq};

use std::string::ToString;

pub struct Session {
    config: LazyCell<Config>,
    catalog: LazyCell<Catalog>,
    project: Option<Project>
}

impl Session {

    pub fn new() -> Result<Session, failure::Error> {
        Ok(Session {
            config: LazyCell::new(),
            catalog: LazyCell::new(),
            project: Project::for_current_dir()?
        })
    }

    pub fn catalog(&self) -> Result<&Catalog, failure::Error> {
        self.catalog.try_borrow_with(|| Catalog::current())
    }

    pub fn catalog_mut(&mut self) -> Result<&mut Catalog, failure::Error> {
        self.catalog.try_borrow_mut_with(|| Catalog::current())
    }

    pub fn config(&self) -> Result<&Config, failure::Error> {
        self.config.try_borrow_with(|| config::config())
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
            //let req: VersionReq = project.manifest().node_req();
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
                panic!("there's no plugin")
            }
        }
    }

}
