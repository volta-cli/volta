//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout.

use std::path::{Path, PathBuf};

cfg_if! {
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

pub fn node_distro_file_name(version: &str) -> String {
    format!("{}.{}", node_archive_root_dir_name(version), archive_extension())
}

pub fn node_npm_version_file_name(version: &str) -> String {
    format!("node-v{}-npm", version)
}

pub fn node_archive_root_dir_name(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&node_archive_root_dir_name(version))
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("package.json")
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
        assert_eq!(
            yarn_distro_file_name("1.2.3"),
            "yarn-v1.2.3.tar.gz"
        );
    }

    #[test]
    fn yarn_node_archive_root_dir() {
        assert_eq!(yarn_archive_root_dir_name("1.2.3"), "yarn-v1.2.3".to_string());
    }
}
