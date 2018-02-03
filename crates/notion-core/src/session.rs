use config::{self, Config};
use catalog::Catalog;
use project::Project;
use version::Version;
use failure;
use lazycell::LazyCell;

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

    pub fn config(&self) -> Result<&Config, failure::Error> {
        self.config.try_borrow_with(|| config::config())
    }

    pub fn node_version(&self) -> Result<Option<String>, failure::Error> {
        if let Some(ref project) = self.project {
            return Ok(Some(project.lockfile()?.node.version.clone()));
        }

        match self.catalog()?.node {
            Some(Version::Public(ref version)) => {
                Ok(Some(version.clone()))
            }
            None => {
                Ok(None)
            }
        }
    }

}
