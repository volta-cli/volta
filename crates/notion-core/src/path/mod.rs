//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use notion_fail::{Fallible, ResultExt};

cfg_if::cfg_if! {
    if #[cfg(feature = "universal-docs")] {
        #[doc(cfg(unix))]
        mod unix;

        #[doc(cfg(windows))]
        mod windows;

        pub use self::unix::*;
    } else if #[cfg(unix)] {
        mod unix;
        pub use self::unix::*;
    } else {
        mod windows;
        pub use self::windows::*;
    }
}

pub fn ensure_notion_dirs_exist() -> Fallible<()> {
    // Assume that if notion_home() exists, then the directory structure has been initialized
    if !notion_home()?.exists() {
        fs::create_dir_all(node_cache_dir()?).unknown()?;
        fs::create_dir_all(shim_dir()?).unknown()?;
        fs::create_dir_all(node_inventory_dir()?).unknown()?;
        fs::create_dir_all(package_inventory_dir()?).unknown()?;
        fs::create_dir_all(yarn_inventory_dir()?).unknown()?;
        fs::create_dir_all(node_image_root_dir()?).unknown()?;
        fs::create_dir_all(yarn_image_root_dir()?).unknown()?;
        fs::create_dir_all(user_toolchain_dir()?).unknown()?;
        fs::create_dir_all(tmp_dir()?).unknown()?;
    }

    Ok(())
}

pub fn notion_home() -> Fallible<PathBuf> {
    if let Some(home) = env::var_os("NOTION_HOME") {
        Ok(Path::new(&home).to_path_buf())
    } else {
        default_notion_home()
    }
}

pub fn cache_dir() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("cache"))
}

pub fn tmp_dir() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("tmp"))
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

pub fn node_cache_dir() -> Fallible<PathBuf> {
    Ok(cache_dir()?.join("node"))
}

pub fn node_index_file() -> Fallible<PathBuf> {
    Ok(node_cache_dir()?.join("index.json"))
}

pub fn node_index_expiry_file() -> Fallible<PathBuf> {
    Ok(node_cache_dir()?.join("index.json.expires"))
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

pub fn yarn_image_root_dir() -> Fallible<PathBuf> {
    Ok(image_dir()?.join("yarn"))
}

pub fn yarn_image_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_image_root_dir()?.join(version))
}

pub fn yarn_image_bin_dir(version: &str) -> Fallible<PathBuf> {
    Ok(yarn_image_dir(version)?.join("bin"))
}

pub fn shim_dir() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("bin"))
}

pub fn user_hooks_file() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("hooks.toml"))
}

pub fn tools_dir() -> Fallible<PathBuf> {
    Ok(notion_home()?.join("tools"))
}

pub fn inventory_dir() -> Fallible<PathBuf> {
    Ok(tools_dir()?.join("inventory"))
}

pub fn user_toolchain_dir() -> Fallible<PathBuf> {
    Ok(tools_dir()?.join("user"))
}

pub fn user_platform_file() -> Fallible<PathBuf> {
    Ok(user_toolchain_dir()?.join("platform.json"))
}

pub fn node_distro_file_name(version: &str) -> String {
    format!(
        "{}.{}",
        node_archive_root_dir_name(version),
        archive_extension()
    )
}

pub fn node_npm_version_file(version: &str) -> Fallible<PathBuf> {
    let filename = format!("node-v{}-npm", version);
    Ok(node_inventory_dir()?.join(&filename))
}

pub fn node_archive_root_dir_name(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}

pub fn yarn_distro_file_name(version: &str) -> String {
    format!("{}.tar.gz", yarn_archive_root_dir_name(version))
}

pub fn yarn_archive_root_dir_name(version: &str) -> String {
    format!("yarn-v{}", version)
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_node_distro_file_name() {
        assert_eq!(
            node_distro_file_name("1.2.3"),
            format!("node-v1.2.3-{}-{}.{}", OS, ARCH, archive_extension())
        );
    }

    #[test]
    fn test_node_archive_root_dir() {
        assert_eq!(
            node_archive_root_dir_name("1.2.3"),
            format!("node-v1.2.3-{}-{}", OS, ARCH)
        );
    }

    #[test]
    fn test_yarn_distro_file_name() {
        assert_eq!(yarn_distro_file_name("1.2.3"), "yarn-v1.2.3.tar.gz");
    }

    #[test]
    fn yarn_node_archive_root_dir() {
        assert_eq!(
            yarn_archive_root_dir_name("1.2.3"),
            "yarn-v1.2.3".to_string()
        );
    }
}
