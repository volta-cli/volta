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
//             binstub.exe                             binstub_file

fn program_data_root() -> Option<PathBuf> {
    winfolder::known_path(&winfolder::id::PROGRAM_DATA).map(|pd| {
        pd.join("Nodeup")
    })
}

pub fn cache_dir() -> Option<PathBuf> {
    program_data_root().map(|root| {
        root.join("cache")
    })
}

pub fn node_cache_dir() -> Option<PathBuf> {
    cache_dir().map(|cache| {
        cache.join("node")
    })
}

pub fn archive_extension() -> String {
    String::from("zip")
}

pub fn versions_dir() -> Option<PathBuf> {
    program_data_root().map(|root| {
        root.join("versions")
    })
}

pub fn node_versions_dir() -> Option<PathBuf> {
    versions_dir().map(|versions| {
        versions.join("node")
    })
}

pub fn node_version_dir(version: &str) -> Option<PathBuf> {
    node_versions_dir().map(|node| {
        node.join(version)
    })
}

pub fn node_version_bin_dir(version: &str) -> Option<PathBuf> {
    node_version_dir(version)
}

pub fn binstub_file() -> Option<PathBuf> {
    program_data_root().map(|root| {
        root.join("binstub.exe")
    })
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

fn program_files_root() -> Option<PathBuf> {
    winfolder::known_path(&winfolder::id::PROGRAM_FILES_X64).map(|pf| {
        pf.join("Nodeup")
    })
}

pub fn bin_dir() -> Option<PathBuf> {
    program_files_root()
}

pub fn nodeup_file() -> Option<PathBuf> {
    bin_dir().map(|bin| {
        bin.join("nodeup.exe")
    })
}

pub fn toolchain_dir() -> Option<PathBuf> {
    program_files_root().map(|root| {
        root.join("toolchain")
    })
}

pub fn toolchain_file(toolname: &str) -> Option<PathBuf> {
    toolchain_dir().map(|toolchain| {
        toolchain.join(&format!("{}.exe", toolname))
    })
}

// C:\
//     Users\
//         dherman\
//             AppData\
//                 Local\
//                     Nodeup\
//                         config.toml                 user_config_file

fn local_data_root() -> Option<PathBuf> {
    winfolder::known_path(&winfolder::id::LOCAL_APP_DATA).map(|adl| {
        adl.join("Nodeup")
    })
}

pub fn user_config_file() -> Option<PathBuf> {
    local_data_root().map(|root| {
        root.join("config.toml")
    })
}
