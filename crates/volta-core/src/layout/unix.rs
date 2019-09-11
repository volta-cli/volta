use std::path::PathBuf;

use super::volta_home;
use crate::error::ErrorDetails;
use volta_fail::Fallible;

pub(super) fn default_home_dir() -> Fallible<PathBuf> {
    let mut home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
    home.push(".volta");
    Ok(home)
}

pub(super) fn default_install_dir() -> Fallible<PathBuf> {
    // default location for the install directory
    // (this will be the case for the majority of installs)
    let home = volta_home()?.root();
    if home.join("shim").exists() {
        return Ok(home.to_owned());
    }

    // when an RPM is installed as root, the install_dir will be here for non-root users
    // (this will be the case for some managed installs)
    let rpm_home = PathBuf::from("/usr/bin/volta-lib");
    if rpm_home.join("shim").exists() {
        return Ok(rpm_home);
    }

    Err(ErrorDetails::ShimExecutableNotFound.into())
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    let home = volta_home()?;
    Ok(vec![home.shim_dir().to_owned()])
}
