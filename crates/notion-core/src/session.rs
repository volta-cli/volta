use config::{self, Config, NodeConfig, Plugin};
use catalog::Catalog;
use project::Project;
use failure;
use lazycell::LazyCell;
use semver::{Version, VersionReq};

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
        let catalog = self.catalog()?;

        if let Some(ref project) = self.project {
            let req: VersionReq = project.manifest().node_req();
            let available = catalog.node.resolve_local(&req);

            return if available.is_some() {
                Ok(available)
            } else {
                self.resolve_remote_node(&req).map(Some)
            }
        }

        Ok(catalog.node.current.clone())
    }

    fn resolve_remote_node(&self, req: &VersionReq) -> Result<Version, failure::Error> {
        let config = self.config()?;

        match config.node {
            Some(NodeConfig { resolve: Some(Plugin::Url(_)), .. }) => {
                unimplemented!()
            }
            Some(NodeConfig { resolve: Some(Plugin::Bin(ref bin)), .. }) => {
                panic!("there's a bin plugin")
            }
            _ => {
                panic!("there's no plugin")
            }
        }
    }

}
