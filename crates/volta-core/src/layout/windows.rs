use std::path::PathBuf;

use super::{volta_home, volta_install};
use crate::error::{ErrorKind, Fallible};

pub(super) fn default_home_dir() -> Fallible<PathBuf> {
    let mut home = dirs::data_local_dir().ok_or(ErrorKind::NoLocalDataDir)?;
    home.push("Volta");
    Ok(home)
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    let home = volta_home()?;
    let install = volta_install()?;

    Ok(vec![home.shim_dir().to_owned(), install.root().to_owned()])
}
