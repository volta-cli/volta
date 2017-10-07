use std::fs::remove_dir_all;

use config;

// FIXME: should return Option<Result<...>>
pub fn by_version(version: &str) -> Option<()> {
    let home = config::node_version_dir(version).unwrap();

    if !home.is_dir() {
        return None;
    }

    remove_dir_all(home).unwrap();
    Some(())
}
