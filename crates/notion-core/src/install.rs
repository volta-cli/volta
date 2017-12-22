use provision;
use path;

pub enum Installed {
    Previously,
    Now
}

pub fn by_version(version: &str) -> ::Result<Installed> {
    if path::node_version_dir(version)?.is_dir() {
        Ok(Installed::Previously)
    } else {
        let dest = path::node_versions_dir()?;
        provision::by_version(&dest, version)?;
        Ok(Installed::Now)
    }
}
