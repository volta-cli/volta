use global;
use version::Version;
use project::Project;
use failure;

pub fn local() -> Result<Option<String>, failure::Error> {
    match Project::for_current_dir()? {
        Some(mut project) => {
            Ok(Some(project.lockfile()?.node.version.clone()))
        }
        None => Ok(None)
    }
}

pub fn global() -> Result<Option<String>, failure::Error> {
    let state = global::state()?;
    Ok(state.node.map(|Version::Public(version)| version))
}

pub fn both() -> Result<(Option<String>, Option<String>), failure::Error> {
    Ok((local()?, global()?))
}
