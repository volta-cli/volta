use std::path::PathBuf;

use super::{volta_home, volta_install};
use crate::error::ErrorDetails;
use cfg_if::cfg_if;
use volta_fail::Fallible;

pub(super) fn default_home_dir() -> Fallible<PathBuf> {
    let mut home = dirs::data_local_dir().ok_or(ErrorDetails::NoLocalDataDir)?;
    home.push("Volta");
    Ok(home)
}

cfg_if! {
    if #[cfg(test)] {
        pub(super) fn default_install_dir() -> Fallible<PathBuf> {
            // None of the current tests require that this be a directory that exists, but there are
            // some that need this function to not result in an error. Since the tests are run on
            // machines that might not have the Volta keys in the registry, use a dummy value
            Ok(PathBuf::from(r#"Z:\"#))
        }
    } else {
        use std::io;
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};
        use volta_fail::ResultExt;

        // This path needs to exactly match the Registry Key in the Windows Installer
        // wix/main.wxs -
        const VOLTA_REGISTRY_PATH: &'static str = r#"Software\The Volta Maintainers\Volta"#;

        // This Key needs to exactly match the Name from the above element in the Windows Installer
        const VOLTA_INSTALL_DIR: &'static str = "InstallDir";

        pub(super) fn default_install_dir() -> Fallible<PathBuf> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let volta_key = hklm.open_subkey(VOLTA_REGISTRY_PATH).with_context(install_dir_error)?;
            let install_path: String = volta_key.get_value(VOLTA_INSTALL_DIR).with_context(install_dir_error)?;
            Ok(PathBuf::from(install_path))
        }
        fn install_dir_error(_err: &io::Error) -> ErrorDetails {
            ErrorDetails::NoInstallDir
        }
    }
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    let home = volta_home()?;
    let install = volta_install()?;
    Ok(vec![home.shim_dir().to_owned(), install.bin_dir()])
}
