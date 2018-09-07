//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout in Windows operating systems.

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

// C:\
//     ProgramData\
//         Notion\
//             cache\                                  cache_dir
//                 node\                               node_cache_dir
//                     node-v4.8.4-win-x64.zip         archive_file("4.8.4")
//                     node-v6.11.3-win-x64.zip
//                     node-v8.6.0-win-x64.zip
//                     ...
//             versions\                               versions_dir
//                 node\                               node_versions_dir
//                     4.8.4\                          node_version_dir("4.8.4")
//                                                     node_version_bin_dir("4.8.4")
//                     6.11.3\
//                     8.6.0\
//                     ...
//             launchbin.exe                           launchbin_file
//             launchscript.exe                        launchscript_file

fn program_data_root() -> Fallible<PathBuf> {
    #[cfg(windows)]
    return Ok(winfolder::Folder::ProgramData.path().join("Notion"));

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}

pub fn cache_dir() -> Fallible<PathBuf> {
    Ok(program_data_root()?.join("cache"))
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

pub fn versions_dir() -> Fallible<PathBuf> {
    Ok(program_data_root()?.join("versions"))
}

pub fn node_versions_dir() -> Fallible<PathBuf> {
    Ok(versions_dir()?.join("node"))
}

pub fn yarn_versions_dir() -> Fallible<PathBuf> {
    Ok(versions_dir()?.join("yarn"))
}

pub fn node_version_dir(version: &str) -> Fallible<PathBuf> {
    Ok(node_versions_dir()?.join(version))
}

pub fn yarn_version_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_versions_dir()?.join(version))
}

pub fn node_version_bin_dir(version: &str) -> Fallible<PathBuf> {
    node_version_dir(version)
}

pub fn yarn_version_bin_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_version_dir(version)?.join("bin"))
}

// 3rd-party binaries installed globally for this node version
pub fn node_version_3p_bin_dir(version: &str) -> Fallible<PathBuf> {
    // ISSUE (#90) Figure out where binaries are globally installed on Windows
    unimplemented!("global 3rd party executables not yet implemented for Windows")
}

pub fn launchbin_file() -> Fallible<PathBuf> {
    Ok(program_data_root()?.join("launchbin.exe"))
}

pub fn launchscript_file() -> Fallible<PathBuf> {
    Ok(program_data_root()?.join("launchscript.exe"))
}

// C:\
//     Program Files\
//         Notion\
//             notion.exe                              notion_file
//             bin\                                    shim_dir
//                 node.exe                            shim_file("node")
//                 npm.exe
//                 npx.exe
//                 ...

fn program_files_root() -> Fallible<PathBuf> {
    #[cfg(all(windows, target_arch = "x86"))]
    return Ok(winfolder::Folder::ProgramFiles.path().join("Notion"));

    #[cfg(all(windows, target_arch = "x86_64"))]
    return Ok(winfolder::Folder::ProgramFilesX64.path().join("Notion"));

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}

pub fn notion_file() -> Fallible<PathBuf> {
    Ok(program_files_root()?.join("notion.exe"))
}

pub fn shim_dir() -> Fallible<PathBuf> {
    Ok(program_files_root()?.join("bin"))
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(&format!("{}.exe", toolname)))
}

// C:\
//     Users\
//         dherman\
//             AppData\
//                 Local\
//                     Notion\
//                         config.toml                 user_config_file
//                         catalog.toml                user_catalog_file

fn local_data_root() -> Fallible<PathBuf> {
    #[cfg(windows)]
    return Ok(winfolder::Folder::LocalAppData.path().join("Notion"));

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}

pub fn user_config_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("config.toml"))
}

pub fn user_catalog_file() -> Fallible<PathBuf> {
    Ok(local_data_root()?.join("catalog.toml"))
}

pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    #[cfg(windows)]
    return windows::fs::symlink_file(src, dst);

    // "universal-docs" is built on a Unix machine, so we can't include Windows-specific libs
    #[cfg(feature = "universal-docs")]
    unimplemented!()
}
