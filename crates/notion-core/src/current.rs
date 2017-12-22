use global;
use version::Version;
use project::Project;

pub fn local() -> ::Result<Option<String>> {
    match Project::for_current_dir()? {
        Some(mut project) => {
            Ok(Some(project.lockfile()?.node.version.clone()))
        }
        None => Ok(None)
    }
}

pub fn global() -> ::Result<Option<String>> {
    let state = global::state()?;
    Ok(state.node.map(|Version::Public(version)| version))
}

pub fn both() -> ::Result<(Option<String>, Option<String>)> {
    Ok((local()?, global()?))
}
