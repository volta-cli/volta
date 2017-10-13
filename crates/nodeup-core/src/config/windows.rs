use std::path::PathBuf;

use winfolder;

pub const OS: &'static str = "win";

// FIXME: also add support for 32-bit or refuse to build for 32-bit target_arch
pub const ARCH: &'static str = "x64";

// C:\
//     ProgramData\
//         Nodeup\
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

fn program_data_root() -> ::Result<PathBuf> {
    let pd = winfolder::known_path(&winfolder::id::PROGRAM_DATA)
        .ok_or_else(|| { ::ErrorKind::UnknownSystemFolder(String::from("PROGRAM_DATA")) })?;
    Ok(pd.join("Nodeup"))
}

pub fn cache_dir() -> ::Result<PathBuf> {
    Ok(program_data_root()?.join("cache"))
}

pub fn node_cache_dir() -> ::Result<PathBuf> {
    Ok(cache_dir()?.join("node"))
}

pub fn archive_extension() -> String {
    String::from("zip")
}

pub fn versions_dir() -> ::Result<PathBuf> {
    Ok(program_data_root()?.join("versions"))
}

pub fn node_versions_dir() -> ::Result<PathBuf> {
    Ok(versions_dir()?.join("node"))
}

pub fn node_version_dir(version: &str) -> ::Result<PathBuf> {
    Ok(node_versions_dir()?.join(version))
}

pub fn node_version_bin_dir(version: &str) -> ::Result<PathBuf> {
    node_version_dir(version)
}

pub fn launchbin_file() -> ::Result<PathBuf> {
    Ok(program_data_root()?.join("launchbin.exe"))
}

pub fn launchscript_file() -> ::Result<PathBuf> {
    Ok(program_data_root()?.join("launchscript.exe"))
}

// C:\
//     Program Files\
//         Nodeup\                                     bin_dir
//             nodeup.exe                              nodeup_file
//             toolchain\                              toolchain_dir
//                 node.exe                            toolchain_file("node")
//                 npm.exe
//                 npx.exe
//                 ...

fn program_files_root() -> ::Result<PathBuf> {
    let pf = winfolder::known_path(&winfolder::id::PROGRAM_FILES_X64)
        .ok_or_else(|| { ::ErrorKind::UnknownSystemFolder(String::from("PROGRAM_FILES_X64")) })?;
    Ok(pf.join("Nodeup"))
}

pub fn bin_dir() -> ::Result<PathBuf> {
    program_files_root()
}

pub fn nodeup_file() -> ::Result<PathBuf> {
    Ok(bin_dir()?.join("nodeup.exe"))
}

pub fn toolchain_dir() -> ::Result<PathBuf> {
    Ok(program_files_root()?.join("toolchain"))
}

pub fn toolchain_file(toolname: &str) -> ::Result<PathBuf> {
    Ok(toolchain_dir()?.join(&format!("{}.exe", toolname)))
}

// C:\
//     Users\
//         dherman\
//             AppData\
//                 Local\
//                     Nodeup\
//                         config.toml                 user_config_file

fn local_data_root() -> ::Result<PathBuf> {
    let adl = winfolder::known_path(&winfolder::id::LOCAL_APP_DATA)
        .ok_or_else(|| { ::ErrorKind::UnknownSystemFolder(String::from("LOCAL_APP_DATA")) })?;
    Ok(adl.join("Nodeup"))
}

pub fn user_config_file() -> ::Result<PathBuf> {
    Ok(local_data_root()?.join("config.toml"))
}
