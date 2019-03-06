//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout in Unix-based operating systems.

use std::io;
use std::os::unix;
use std::path::{Path, PathBuf};

use dirs;

use crate::error::ErrorDetails;
use notion_fail::Fallible;

use super::{node_archive_root_dir_name, node_image_dir, notion_home, shim_dir};

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `path::archive_root_dir` to determine the root directory of the
// contents of a Node installer archive.

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        /// The OS component of a Node distribution tarball's name.
        pub const OS: &'static str = "darwin";
    } else if #[cfg(target_os = "linux")] {
        /// The OS component of a Node distribution tarball's name.
        pub const OS: &'static str = "linux";
    } else {
        compile_error!("Unsupported target_os variant of unix (expected 'macos' or 'linux').");
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86")] {
        /// The system architecture component of a Node distribution tarball's name.
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        /// The system architecture component of a Node distribution tarball's name.
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant of unix (expected 'x86' or 'x64').");
    }
}

// ~/
//     .notion/
//         cache/                                          cache_dir
//             node/                                       node_cache_dir
//                 index.json                              node_index_file
//                 index.json.expires                      node_index_expiry_file
//         bin/                                            shim_dir
//             node                                        shim_file("node")
//             npm
//             npx
//             ...
//         tools/                                          tools_dir
//             inventory/                                  inventory_dir
//                 node/                                   node_inventory_dir
//                     node-v4.8.4-linux-x64.tar.gz        node_distro_file_name("4.8.4")
//                     node-v4.8.4-npm                     node_npm_version_file("4.8.4")
//                     ...
//                 packages/                               package_inventory_dir
//                 yarn/                                   yarn_inventory_dir
//             image/                                      image_dir
//                 node/                                   node_image_root_dir
//                     10.13.0/
//                         6.4.0/                          node_image_dir("10.13.0", "6.4.0")
//                             bin/                        node_image_bin_dir("10.13.0", "6.4.0")
//                 yarn/                                   yarn_image_root_dir
//                     1.7.0/                              yarn_image_dir("1.7.0")
//             user/                                       user_toolchain_dir
//                 bins/
//                     ember ~> ../packages/ember-cli
//                 packages/
//                     ember-cli/
//                         package.toml
//                         contents/
//                 platform.json                           user_platform_file
//         notion                                          notion_file
//         shim                                            shim_file
//         hooks.toml                                      user_hooks_file

pub fn default_notion_home() -> Fallible<PathBuf> {
    let home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
    Ok(home.join(".notion"))
}

pub fn archive_extension() -> String {
    String::from("tar.gz")
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(node_image_dir(node, npm)?.join("bin"))
}

// 3rd-party binaries installed globally for this node version
pub fn node_image_3p_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(node_image_dir(node, npm)?.join("lib/node_modules/.bin"))
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&node_archive_root_dir_name(version))
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("package.json")
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(toolname))
}

pub fn notion_file() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("notion"))
}

pub fn shim_executable() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("shim"))
}

pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    unix::fs::symlink(src, dst)
}
