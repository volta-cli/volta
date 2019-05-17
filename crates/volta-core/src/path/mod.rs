//! Provides functions for determining the paths of files and directories
//! in a standard Volta layout.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ErrorDetails;
use volta_fail::{Fallible, ResultExt};

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

pub fn ensure_volta_dirs_exist() -> Fallible<()> {
    // Assume that if volta_home() exists, then the directory structure has been initialized
    if !volta_home()?.exists() {
        ensure_dir_exists(node_cache_dir()?)?;
        ensure_dir_exists(shim_dir()?)?;
        ensure_dir_exists(node_inventory_dir()?)?;
        ensure_dir_exists(package_inventory_dir()?)?;
        ensure_dir_exists(yarn_inventory_dir()?)?;
        ensure_dir_exists(node_image_root_dir()?)?;
        ensure_dir_exists(yarn_image_root_dir()?)?;
        ensure_dir_exists(user_toolchain_dir()?)?;
        ensure_dir_exists(tmp_dir()?)?;
        ensure_dir_exists(log_dir()?)?;
    }

    Ok(())
}

fn ensure_dir_exists(path: PathBuf) -> Fallible<()> {
    fs::create_dir_all(&path).with_context(|_| ErrorDetails::CreateDirError { dir: path })
}

pub fn volta_home() -> Fallible<PathBuf> {
    if let Some(home) = env::var_os("VOLTA_HOME") {
        Ok(Path::new(&home).to_path_buf())
    } else {
        default_volta_home()
    }
}

pub fn cache_dir() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("cache"))
}

pub fn tmp_dir() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("tmp"))
}

pub fn log_dir() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("log"))
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

pub fn package_distro_file(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(package_inventory_dir()?.join(package_distro_file_name(name, version)))
}

pub fn package_distro_shasum(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(package_inventory_dir()?.join(package_shasum_file_name(name, version)))
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

pub fn package_image_root_dir() -> Fallible<PathBuf> {
    Ok(image_dir()?.join("packages"))
}

pub fn package_image_dir(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(package_image_root_dir()?.join(name).join(version))
}

pub fn shim_dir() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("bin"))
}

pub fn user_hooks_file() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("hooks.json"))
}

pub fn tools_dir() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("tools"))
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

pub fn user_package_dir() -> Fallible<PathBuf> {
    Ok(user_toolchain_dir()?.join("packages"))
}

pub fn user_package_config_file(package_name: &str) -> Fallible<PathBuf> {
    Ok(user_package_dir()?.join(format!("{}.json", package_name)))
}

pub fn user_bin_dir() -> Fallible<PathBuf> {
    Ok(user_toolchain_dir()?.join("bins"))
}

pub fn user_tool_bin_config(bin_name: &str) -> Fallible<PathBuf> {
    Ok(user_bin_dir()?.join(format!("{}.json", bin_name)))
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

pub fn package_distro_file_name(name: &str, version: &str) -> String {
    format!("{}.tgz", package_archive_root_dir_name(name, version))
}

pub fn package_shasum_file_name(name: &str, version: &str) -> String {
    format!("{}.shasum", package_archive_root_dir_name(name, version))
}

pub fn package_archive_root_dir_name(name: &str, version: &str) -> String {
    format!("{}-{}", name, version)
}

fn is_node_root(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn is_node_modules(dir: &Path) -> bool {
    dir.file_name() == Some(OsStr::new("node_modules"))
}

fn is_dependency(dir: &Path) -> bool {
    dir.parent().map_or(false, |parent| is_node_modules(parent))
}

fn is_project_root(dir: &Path) -> bool {
    is_node_root(dir) && !is_dependency(dir)
}

pub fn find_project_dir(base_dir: &Path) -> Option<&Path> {
    let mut dir = base_dir.clone();
    while !is_project_root(dir) {
        dir = match dir.parent() {
            Some(parent) => parent,
            None => {
                return None;
            }
        }
    }

    Some(dir)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn fixture_path(fixture_dirs: &[&str]) -> PathBuf {
        let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_manifest_dir.push("fixtures");

        for fixture_dir in fixture_dirs.iter() {
            cargo_manifest_dir.push(fixture_dir);
        }

        cargo_manifest_dir
    }

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

    #[test]
    fn test_find_project_dir_direct() {
        let base_dir = fixture_path(&["basic"]);
        let project_dir = find_project_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, base_dir);
    }

    #[test]
    fn test_find_project_dir_ancestor() {
        let base_dir = fixture_path(&["basic", "subdir"]);
        let project_dir = find_project_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }

    #[test]
    fn test_find_project_dir_dependency() {
        let base_dir = fixture_path(&["basic", "node_modules", "eslint"]);
        let project_dir = find_project_dir(&base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }
}
