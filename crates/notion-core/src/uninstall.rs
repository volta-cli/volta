use std::fs::remove_dir_all;
use std::io;

use failure;

use path;

pub fn by_version(version: &str) -> Result<(), failure::Error> {
    let home = path::node_version_dir(version)?;

    if !home.is_dir() {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", home.to_string_lossy())))?;
    }

    remove_dir_all(home)?;
    Ok(())
}
