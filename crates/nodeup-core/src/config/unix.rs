use std::env;
use std::path::PathBuf;

// FIXME: make the case analysis here complete and rigorous

#[cfg(target_os = "macos")]
pub const OS: &'static str = "darwin";

#[cfg(target_os = "linux")]
pub const OS: &'static str = "linux";

#[cfg(target_arch = "x86")]
pub const ARCH: &'static str = "x86";

#[cfg(target_arch = "x86_64")]
pub const ARCH: &'static str = "x64";

// ~/
//     .nodeup/
//         cache/                                          cache_dir
//             node/                                       node_cache_dir
//                 node-dist-v4.8.4-linux-x64.tar.gz       archive_file("4.8.4")
//                 node-dist-v6.11.3-linux-x64.tar.gz
//                 node-dist-v8.6.0-linux-x64.tar.gz
//                 ...
//         versions/                                       versions_dir
//             node/                                       node_versions_dir
//                 4.8.4/                                  node_version_dir("4.8.4")
//                   bin/                                  node_version_bin_dir("4.8.4")
//                 6.11.3/
//                 8.6.0/
//                 ...
//         bin/                                            bin_dir
//             nodeup                                      nodeup_file
//         toolchain/                                      toolchain_dir
//             node                                        toolchain_file("node")
//             npm
//             npx
//             ...
//         binstub                                         binstub_file
//         scriptstub                                      scriptstub_file
//         config.toml                                     user_config_file

fn nodeup_home() -> Option<PathBuf> {
    env::home_dir().map(|home| {
        home.join(".nodeup")
    })
}

pub fn cache_dir() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("cache")
    })
}

pub fn node_cache_dir() -> Option<PathBuf> {
    cache_dir().map(|cache| {
        cache.join("node")
    })
}

pub fn archive_extension() -> String {
    String::from("tar.gz")
}

pub fn versions_dir() -> Option<PathBuf> {
    nodeup_home().map(|root| {
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
    node_version_dir(version).map(|node| {
        node.join("bin")
    })
}

pub fn bin_dir() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("bin")
    })
}

pub fn nodeup_file() -> Option<PathBuf> {
    bin_dir().map(|bin| {
        bin.join("nodeup")
    })
}

pub fn toolchain_dir() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("toolchain")
    })
}

pub fn toolchain_file(toolname: &str) -> Option<PathBuf> {
    toolchain_dir().map(|toolchain| {
        toolchain.join(toolname)
    })
}

pub fn binstub_file() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("binstub")
    })
}

pub fn scriptstub_file() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("scriptstub")
    })
}

pub fn user_config_file() -> Option<PathBuf> {
    nodeup_home().map(|root| {
        root.join("config.toml")
    })
}
