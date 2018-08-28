//! Provides functions for determining the paths of files and directories
//! in a standard Notion layout.

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

pub fn node_archive_file(version: &str) -> String {
    format!("{}.{}", node_archive_root_dir(version), archive_extension())
}

pub fn node_archive_root_dir(version: &str) -> String {
    format!("node-v{}-{}-{}", version, OS, ARCH)
}

pub fn yarn_archive_file(version: &str) -> String {
    format!("{}.{}", yarn_archive_root_dir(version), archive_extension())
}

pub fn yarn_archive_root_dir(version: &str) -> String {
    format!("yarn-v{}", version)
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_node_archive_file() {
        assert_eq!(
            node_archive_file("1.2.3"),
            format!("node-v1.2.3-{}-{}.{}", OS, ARCH, archive_extension())
        );
    }

    #[test]
    fn test_node_archive_root_dir() {
        assert_eq!(
            node_archive_root_dir("1.2.3"),
            format!("node-v1.2.3-{}-{}", OS, ARCH)
        );
    }

    #[test]
    fn yarn_node_archive_file() {
        assert_eq!(
            yarn_archive_file("1.2.3"),
            format!("yarn-v1.2.3.{}", archive_extension())
        );
    }

    #[test]
    fn yarn_node_archive_root_dir() {
        assert_eq!(yarn_archive_root_dir("1.2.3"), "yarn-v1.2.3".to_string());
    }
}
