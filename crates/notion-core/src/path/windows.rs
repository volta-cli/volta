//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout in Windows operating systems.

use std::env;
use std::path::PathBuf;
#[cfg(windows)]
use std::os::windows;
use std::io;

use winfolder;

use notion_fail::Fallible;

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `path::archive_root_dir` to determine the root directory of the
// contents of a Node installer archive.

pub const OS: &'static str = "win";

cfg_if! {
    if #[cfg(target_arch = "x86")] {
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant of Windows (expected 'x86' or 'x64').");
    }
}

// C:\Users\dherman\AppData\Local\
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
//                 platform.toml                           user_platform_file
//         notion.exe                                      notion_file
//         launchbin.exe                                   launchbin_file
//         launchscript.exe                                launchscript_file
//         config.toml                                     user_config_file

pub fn cache_dir() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("cache"))
}

pub fn node_cache_dir() -> Fallible<PathBuf> {
    Ok(cache_dir()?.join("node"))
}

pub fn yarn_cache_dir() -> Fallible<PathBuf> {
    Ok(cache_dir()?.join("yarn"))
}

pub fn node_index_file() -> Fallible<PathBuf> {
    Ok(node_cache_dir()?.join("index.json"))
}

pub fn node_index_expiry_file() -> Fallible<PathBuf> {
    Ok(node_cache_dir()?.join("index.json.expires"))
}

pub fn archive_extension() -> String {
    String::from("zip")
}

pub fn inventory_dir() -> Fallible<PathBuf> {
    Ok(tools_dir()?.join("inventory"))
}

pub fn node_inventory_dir() -> Fallible<PathBuf> {
    Ok(inventory_dir()?.join("node"))
}

pub fn yarn_inventory_dir() -> Fallible<PathBuf> {
    Ok(inventory_dir()?.join("yarn"))
}

pub fn package_inventory_dir() -> Fallible<PathBuf> {
    Ok(inventory_dir()?.join("packages"))
}

pub fn image_dir() -> Fallible<PathBuf> {
    Ok(tools_dir()?.join("image"))
}

pub fn node_image_root_dir() -> Fallible<PathBuf> {
    Ok(image_dir()?.join("node"))
}

pub fn node_image_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(node_image_root_dir()?.join(node).join(npm))
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    node_image_dir(node, npm)
}

// 3rd-party binaries installed globally for this node version
pub fn node_image_3p_bin_dir(_node: &str, _npm: &str) -> Fallible<PathBuf> {
    // ISSUE (#90) Figure out where binaries are globally installed on Windows
    unimplemented!("global 3rd party executables not yet implemented for Windows")
}

pub fn yarn_image_root_dir() -> Fallible<PathBuf> {
    Ok(image_dir()?.join("yarn"))
}

pub fn yarn_image_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_image_root_dir()?.join(version))
}

pub fn yarn_image_bin_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_image_dir(version)?.join("bin"))
}

pub fn launchbin_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("launchbin.exe"))
}

pub fn launchscript_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("launchscript.exe"))
}

pub fn notion_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("notion.exe"))
}

pub fn shim_dir() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("bin"))
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(&format!("{}.exe", toolname)))
}

fn local_data_root() -> Fallible<PathBuf> {
    // if this is sandboxed in CI, use the sandboxed AppData directory
    if env::var("NOTION_SANDBOX").is_ok() {
        let home_dir = env::home_dir().unwrap();
        return Ok(home_dir.join("AppData").join("Local").join("Notion"));
    } else {
        #[cfg(windows)]
        return Ok(winfolder::Folder::LocalAppData.path().join("Notion"));

        // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
        #[cfg(feature = "universal-docs")]
        unimplemented!()
    }
}

pub fn user_config_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("config.toml"))
}

pub fn tools_dir() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("tools"))
}

pub fn user_toolchain_dir() -> Fallible<PathBuf> {
    Ok(tools_dir()?.join("user"))
}

pub fn user_platform_file() -> Fallible<PathBuf> {
    Ok(user_toolchain_dir()?.join("platform.toml"))
}

pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    #[cfg(windows)]
    return windows::fs::symlink_file(src, dst);

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}
