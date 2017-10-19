use std::fs::remove_dir_all;
use std::io;

use path;

pub fn by_version(version: &str) -> ::Result<()> {
    let home = path::node_version_dir(version)?;

    if !home.is_dir() {
        bail!(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", home.to_string_lossy())));
    }

    remove_dir_all(home)?;
    Ok(())
}
