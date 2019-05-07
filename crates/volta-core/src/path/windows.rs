//! Provides functions for determining the paths of files and directories
//! in a standard Volta layout in Windows operating systems.

use std::io;
#[cfg(windows)]
use std::os::windows;
use std::path::{Path, PathBuf};

use crate::error::ErrorDetails;
use cfg_if::cfg_if;
use dirs;
use volta_fail::{Fallible, ResultExt};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use super::{node_archive_root_dir_name, node_image_dir, shim_dir};

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `path::archive_root_dir` to determine the root directory of the
// contents of a Node installer archive.

pub const OS: &'static str = "win";

// This path needs to exactly match the Registry Key in the Windows Installer
// wix/main.wxs -
const NOTION_REGISTRY_PATH: &'static str = r#"Software\The Volta Maintainers\Volta"#;

// This Key needs to exactly match the Name from the above element in the Windows Installer
const NOTION_INSTALL_DIR: &'static str = "InstallDir";

cfg_if! {
    if #[cfg(target_arch = "x86")] {
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant of Windows (expected 'x86' or 'x64').");
    }
}

// C:\Users\johndoe\AppData\Local\
//     Volta\
//         cache\                                          cache_dir
//             node\                                       node_cache_dir
//                 index.json                              node_index_file
//                 index.json.expires                      node_index_expiry_file
//         bin\                                            shim_dir
//             node.exe                                    shim_file("node")
//             npm.exe
//             npx.exe
//             ...
//         log\                                            log_dir
//         tools\                                          tools_dir
//             inventory\                                  inventory_dir
//                 node\                                   node_inventory_dir
//                     node-v4.8.4-win-x64.zip             node_archive_file("4.8.4")
//                     node-v4.8.4-npm                     node_npm_version_file("4.8.4")
//                     ...
//                 packages\                               package_inventory_dir
//                     ember-cli-3.7.1.tgz                 package_distro_file("ember-cli", "3.7.1")
//                     ember-cli-3.7.1.shasum              package_distro_shasum("ember-cli", "3.7.1")
//                 yarn\                                   yarn_inventory_dir
//             image\                                      image_dir
//                 node\                                   node_image_root_dir
//                     10.13.0\
//                         6.4.0\                          node_image_dir("10.13.0", "6.4.0")
//                                                         node_image_bin_dir("10.13.0", "6.4.0")
//                 yarn\                                   yarn_image_root_dir
//                     1.7.0\                              yarn_image_dir("1.7.0")
//                 packages\                               package_image_root_dir
//                     ember-cli\
//                         3.7.1\                          package_image_dir("ember-cli", "3.7.1")
//             user\                                       user_toolchain_dir
//                 bins\
//                     tsc.json                            user_tool_bin_config("tsc")
//                 packages\                               user_package_dir
//                     ember-cli.json                      user_package_config_file("ember-cli")
//                 platform.json                           user_platform_file
//         hooks.toml                                      user_hooks_file
//
// C:\Program Files\
//     Volta\                                             (Path stored in Windows Registry by installer)
//         bin\
//             volta.exe                                  volta_file
//             node.exe                                    copy of shim_executable
//             npm.exe                                     copy of shim_executable
//             npx.exe                                     copy of shim_executable
//             yarn.exe                                    copy of shim_executable
//         shim.exe                                        shim_executable

pub fn default_volta_home() -> Fallible<PathBuf> {
    let home = dirs::data_local_dir().ok_or(ErrorDetails::NoLocalDataDir)?;
    Ok(home.join("Volta"))
}

pub fn archive_extension() -> String {
    String::from("zip")
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    node_image_dir(node, npm)
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&node_archive_root_dir_name(version))
        .join("node_modules")
        .join("npm")
        .join("package.json")
}

cfg_if::cfg_if! {
    // We don't want to be reading from the Registry when testing, so use a fixture PathBuf
    if #[cfg(test)] {
        fn install_dir() -> Fallible<PathBuf> {
            Ok(PathBuf::from(r#"Z:\"#))
        }
    } else {
        fn install_dir() -> Fallible<PathBuf> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let volta_key = hklm.open_subkey(NOTION_REGISTRY_PATH).with_context(install_dir_error)?;
            let install_path: String = volta_key.get_value(NOTION_INSTALL_DIR).with_context(install_dir_error)?;
            Ok(PathBuf::from(install_path))
        }
        fn install_dir_error(_err: &io::Error) -> ErrorDetails {
            ErrorDetails::NoInstallDir
        }
    }
}

pub fn install_bin_dir() -> Fallible<PathBuf> {
    Ok(install_dir()?.join("bin"))
}

pub fn shim_executable() -> Fallible<PathBuf> {
    Ok(install_dir()?.join("shim.exe"))
}

pub fn volta_file() -> Fallible<PathBuf> {
    Ok(install_bin_dir()?.join("volta.exe"))
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(&format!("{}.exe", toolname)))
}

pub fn shim_git_bash_script_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(toolname))
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    Ok(vec![shim_dir()?, install_bin_dir()?])
}

/// Create a symlink. The `dst` path will be a symbolic link pointing to the `src` path.
pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    #[cfg(windows)]
    return windows::fs::symlink_file(src, dst);

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}
