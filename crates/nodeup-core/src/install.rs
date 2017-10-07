use provision;
use config;

// FIXME: should return Option<Result<...>>
pub fn by_version(version: &str) -> Option<()> {
    if config::node_version_dir(version).unwrap().is_dir() {
        return None;
    }

    let dest = config::node_versions_dir().unwrap();
    provision::by_version(&dest, version);
    Some(())
}
