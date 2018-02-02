use config::{self, Config};
use catalog::Catalog;
use project::Project;
use version::Version;
use failure;
use lazycell::LazyCell;

pub enum Location {
    Global(Catalog),
    Local(Project)
}

impl Location {

    pub fn current() -> Result<Location, failure::Error> {
        Ok(if let Some(project) = Project::for_current_dir()? {
            Location::Local(project)
        } else {
            Location::Global(Catalog::current()?)
        })
    }

    pub fn node_version(&self) -> Result<Option<String>, failure::Error> {
        match self {
            &Location::Global(Catalog { node: None }) => {
                Ok(None)
            }
            &Location::Global(Catalog { node: Some(Version::Public(ref version))}) => {
                Ok(Some(version.clone()))
            }
            &Location::Local(ref project) => {
                Ok(Some(project.lockfile()?.node.version.clone()))
            }
        }
    }

}

pub struct Session {
    config: LazyCell<Config>,
    location: Location
}

impl Session {

    pub fn new() -> Result<Session, failure::Error> {
        let location = Location::current()?;
        Ok(Session {
            config: LazyCell::new(),
            location: location
        })
    }

    pub fn config(&self) -> Result<&Config, failure::Error> {
        self.config.try_borrow_with(|| config::config())
    }

    pub fn node_version(&self) -> Result<Option<String>, failure::Error> {
        self.location.node_version()
    }

}
