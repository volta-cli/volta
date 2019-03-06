//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout in Windows operating systems.

use std::io;
#[cfg(windows)]
use std::os::windows;
use std::path::{Path, PathBuf};

use dirs;

use crate::error::ErrorDetails;
use notion_fail::{Fallible, ResultExt};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use super::{node_archive_root_dir_name, node_image_dir, shim_dir};

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `path::archive_root_dir` to determine the root directory of the
// contents of a Node installer archive.

pub const OS: &'static str = "win";

// This path needs to exactly match the Registry Key in the Windows Installer
// wix/main.wxs -
const NOTION_REGISTRY_PATH: &'static str = r#"Software\The Notion Maintainers\Notion"#;

// This Key needs to exactly match the Name from the above element in the Windows Installer
const NOTION_INSTALL_DIR: &'static str = "InstallDir";

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86")] {
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant of Windows (expected 'x86' or 'x64').");
    }
}

// C:\Users\johndoe\AppData\Local\
//     Notion\
//         cache\                                          cache_dir
//             node\                                       node_cache_dir
//                 index.json                              node_index_file
//                 index.json.expires                      node_index_expiry_file
//         bin\                                            shim_dir
//             node                                        shim_file("node")
//             npm
//             npx
//             ...
//         tools\                                          tools_dir
//             inventory\                                  inventory_dir
//                 node\                                   node_inventory_dir
//                     node-v4.8.4-win-x64.zip             node_archive_file("4.8.4")
//                     node-v4.8.4-npm                     node_npm_version_file("4.8.4")
//                     ...
//                 packages\                               package_inventory_dir
//                 yarn\                                   yarn_inventory_dir
//             image\                                      image_dir
//                 node\                                   node_image_root_dir
//                     10.13.0\
//                         6.4.0\                          node_image_dir("10.13.0", "6.4.0")
//                                                         node_image_bin_dir("10.13.0", "6.4.0")
//                 yarn\                                   yarn_image_root_dir
//                     1.7.0\                              yarn_image_dir("1.7.0")
//             user\                                       user_toolchain_dir
//                 bins\
//                     ember ~> ..\packages\ember-cli
//                 packages\
//                     ember-cli\
//                         package.toml
//                         contents\
//                 platform.json                           user_platform_file
//         hooks.toml                                      user_hooks_file
//
// C:\Program Files\
//     Notion\                                             (Path stored in Windows Registry by installer)
//         notion.exe                                      notion_file
//         launchbin.exe                                   launchbin_file
//         launchscript.exe                                launchscript_file

pub fn default_notion_home() -> Fallible<PathBuf> {
    let home = dirs::data_local_dir().ok_or(ErrorDetails::NoLocalDataDir)?;
    Ok(home.join("Notion"))
}

pub fn archive_extension() -> String {
    String::from("zip")
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    node_image_dir(node, npm)
}

// 3rd-party binaries installed globally for this node version
pub fn node_image_3p_bin_dir(_node: &str, _npm: &str) -> Fallible<PathBuf> {
    // ISSUE (#90) Figure out where binaries are globally installed on Windows
    unimplemented!("global 3rd party executables not yet implemented for Windows")
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&node_archive_root_dir_name(version))
        .join("node_modules")
        .join("npm")
        .join("package.json")
}

fn install_dir() -> Fallible<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let notion_key = hklm.open_subkey(NOTION_REGISTRY_PATH).unknown()?;
    let install_path: String = notion_key.get_value(NOTION_INSTALL_DIR).unknown()?;
    Ok(PathBuf::from(install_path))
}

pub fn shim_executable() -> Fallible<PathBuf> {
    Ok(install_dir()?.join("shim.exe"))
}

pub fn notion_file() -> Fallible<PathBuf> {
    Ok(install_dir()?.join("notion.exe"))
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(&format!("{}.exe", toolname)))
}

pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    #[cfg(windows)]
    return windows::fs::symlink_file(src, dst);

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}
