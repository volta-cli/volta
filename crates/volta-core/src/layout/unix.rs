use std::path::PathBuf;

use super::volta_home;
use crate::error::ErrorDetails;
use volta_fail::Fallible;

pub(super) fn default_home_dir() -> Fallible<PathBuf> {
    let mut home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
    home.push(".volta");
    Ok(home)
}

#[cfg(not(feature = "volta-updates"))]
pub(super) fn default_install_dir() -> Fallible<PathBuf> {
    // default location for the install directory
    // (this will be the case for the majority of installs)
    let home = volta_home()?.root();
    if home.join("shim").exists() {
        return Ok(home.to_owned());
    }

    // The current Volta RPM, when run as root, installs the `volta` binary into `/usr/bin`
    // and the `shim` binary in `/usr/bin/volta-lib`, so check there as well for the shim
    // This logic will be changing with full updates https://github.com/volta-cli/rfcs/pull/37
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
