use provision;
use config;

pub enum Installed {
    Previously,
    Now
}

pub fn by_version(version: &str) -> ::Result<Installed> {
    if config::node_version_dir(version)?.is_dir() {
        Ok(Installed::Previously)
    } else {
        let dest = config::node_versions_dir()?;
        provision::by_version(&dest, version)?;
        Ok(Installed::Now)
    }
}
