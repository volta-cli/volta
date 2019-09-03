//! Provides functions for determining the paths of files and directories
//! in a standard Volta layout.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ErrorDetails;
use crate::shim;
use lazycell::AtomicLazyCell;
use log::debug;
use volta_fail::{Fallible, ResultExt};

#[macro_use]
mod macros;

cfg_if::cfg_if! {
    if #[cfg(feature = "cross-platform-docs")] {
        // Mark in the API docs as Unix-only.
        // https://doc.rust-lang.org/nightly/unstable-book/language-features/doc-cfg.html
        #[doc(cfg(unix))]
        mod unix;

        // Mark in the API docs as Windows-only.
        // https://doc.rust-lang.org/nightly/unstable-book/language-features/doc-cfg.html
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

static VOLTA_HOME: AtomicLazyCell<PathBuf> = AtomicLazyCell::NONE;
static INSTALL_DIR: AtomicLazyCell<PathBuf> = AtomicLazyCell::NONE;

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
        // also ensure the basic shims exist
        // this is only for unix until the update process is refactored
        // (windows stores the location in the Registry, which is not available for the tests)
        #[cfg(unix)]
        {
            shim::create("node")?;
            shim::create("yarn")?;
            shim::create("npm")?;
            shim::create("npx")?;
        }
    }

    Ok(())
}

fn ensure_dir_exists(path: PathBuf) -> Fallible<()> {
    fs::create_dir_all(&path).with_context(|_| ErrorDetails::CreateDirError { dir: path })
}

pub fn volta_home() -> Fallible<PathBuf> {
    if let Some(path) = VOLTA_HOME.borrow() {
        return Ok(path.clone());
    }

    let home = match env::var_os("VOLTA_HOME") {
        Some(path) => PathBuf::from(path),
        None => default_volta_home()?,
    };

    if VOLTA_HOME.fill(home.clone()).is_err() {
        debug!("VOLTA_HOME was filled while it was being calculated");
    }

    Ok(home)
}

pub fn install_dir() -> Fallible<PathBuf> {
    if let Some(path) = INSTALL_DIR.borrow() {
        return Ok(path.clone());
    }

    // if VOLTA_INSTALL_DIR is set, try that first
    // (not documented yet, as it's currently only used for testing)
    let install = match env::var_os("VOLTA_INSTALL_DIR") {
        Some(path) => PathBuf::from(path),
        None => default_install_dir()?,
    };

    if INSTALL_DIR.fill(install.clone()).is_err() {
        debug!("INSTALL_DIR was filled while it was being calculated");
    }

    Ok(install)
}

pub fn cache_dir() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "cache"))
}

pub fn tmp_dir() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "tmp"))
}

pub fn log_dir() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "log"))
}

pub fn node_inventory_dir() -> Fallible<PathBuf> {
    Ok(path_join!(inventory_dir()?, "node"))
}

pub fn yarn_inventory_dir() -> Fallible<PathBuf> {
    Ok(path_join!(inventory_dir()?, "yarn"))
}

pub fn package_inventory_dir() -> Fallible<PathBuf> {
    Ok(path_join!(inventory_dir()?, "packages"))
}

pub fn package_distro_file(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(path_join!(
        package_inventory_dir()?,
        package_distro_file_name(name, version)
    ))
}

pub fn package_distro_shasum(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(path_join!(
        package_inventory_dir()?,
        package_shasum_file_name(name, version)
    ))
}

pub fn node_cache_dir() -> Fallible<PathBuf> {
    Ok(path_join!(cache_dir()?, "node"))
}

pub fn node_index_file() -> Fallible<PathBuf> {
    Ok(path_join!(node_cache_dir()?, "index.json"))
}

pub fn node_index_expiry_file() -> Fallible<PathBuf> {
    Ok(path_join!(node_cache_dir()?, "index.json.expires"))
}

pub fn image_dir() -> Fallible<PathBuf> {
    Ok(path_join!(tools_dir()?, "image"))
}

pub fn node_image_root_dir() -> Fallible<PathBuf> {
    Ok(path_join!(image_dir()?, "node"))
}

pub fn node_image_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(path_join!(node_image_root_dir()?, node, npm))
}

pub fn yarn_image_root_dir() -> Fallible<PathBuf> {
    Ok(path_join!(image_dir()?, "yarn"))
}

pub fn yarn_image_dir(version: &str) -> Fallible<PathBuf> {
    Ok(path_join!(yarn_image_root_dir()?, version))
}

pub fn yarn_image_bin_dir(version: &str) -> Fallible<PathBuf> {
    Ok(path_join!(yarn_image_dir(version)?, "bin"))
}

pub fn package_image_root_dir() -> Fallible<PathBuf> {
    Ok(path_join!(image_dir()?, "packages"))
}

pub fn package_image_dir(name: &str, version: &str) -> Fallible<PathBuf> {
    Ok(path_join!(package_image_root_dir()?, name, version))
}

pub fn shim_dir() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "bin"))
}

pub fn user_hooks_file() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "hooks.json"))
}

pub fn tools_dir() -> Fallible<PathBuf> {
    Ok(path_join!(volta_home()?, "tools"))
}

pub fn inventory_dir() -> Fallible<PathBuf> {
    Ok(path_join!(tools_dir()?, "inventory"))
}

pub fn user_toolchain_dir() -> Fallible<PathBuf> {
    Ok(path_join!(tools_dir()?, "user"))
}

pub fn user_platform_file() -> Fallible<PathBuf> {
    Ok(path_join!(user_toolchain_dir()?, "platform.json"))
}

pub fn user_package_dir() -> Fallible<PathBuf> {
    Ok(path_join!(user_toolchain_dir()?, "packages"))
}

pub fn user_package_config_file(package_name: &str) -> Fallible<PathBuf> {
    Ok(path_join!(
        user_package_dir()?,
        format!("{}.json", package_name)
    ))
}

pub fn user_bin_dir() -> Fallible<PathBuf> {
    Ok(path_join!(user_toolchain_dir()?, "bins"))
}

pub fn user_tool_bin_config(bin_name: &str) -> Fallible<PathBuf> {
    Ok(path_join!(user_bin_dir()?, format!("{}.json", bin_name)))
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
    Ok(path_join!(node_inventory_dir()?, &filename))
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
