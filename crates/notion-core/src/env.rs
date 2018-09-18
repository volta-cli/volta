//! Provides utilities for modifying the environment when a shim calls out to
//! its delegated executable.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use notion_fail::{Fallible, ResultExt};
use path;

pub(crate) fn shell_name() -> Option<String> {
    env::var_os("NOTION_SHELL").map(|s| s.to_string_lossy().into_owned())
}

pub fn postscript_path() -> Option<PathBuf> {
    env::var_os("NOTION_POSTSCRIPT")
        .as_ref()
        .map(|ref s| Path::new(s).to_path_buf())
}

// remove the shim and bin dirs from the path, then prepend any input paths
fn build_path(paths: Vec<PathBuf>) -> Fallible<OsString> {
    let current_dir = env::var_os("PATH").unwrap_or(OsString::new());
    let shim_dir = &path::shim_dir()?;
    let bin_dir = &path::bin_dir()?;
    let split = env::split_paths(&current_dir).filter(|s| s != shim_dir && s != bin_dir);
    let mut path_vec: Vec<PathBuf> = Vec::new();
    for p in paths.iter() {
        path_vec.push(p.to_path_buf());
    }
    path_vec.extend(split);
    env::join_paths(path_vec.iter()).unknown()
}

/// Produces a modified version of the current `PATH` environment variable that
/// will find Node.js executables in the installation directory for the given
/// version of Node instead of in the Notion shim directory.
pub fn path_for_installed_node(version: &str) -> Fallible<OsString> {
    let prepended_paths = vec![path::node_version_bin_dir(version)?];
    build_path(prepended_paths)
}

/// Produces a modified version of the current `PATH` environment variable that
/// will find Yarn executables in the installation directory for the given
/// version of Yarn instead of in the Notion shim directory.
pub fn path_for_installed_yarn(version: &str) -> Fallible<OsString> {
    let prepended_paths = vec![path::yarn_version_bin_dir(version)?];
    build_path(prepended_paths)
}

/// Produces a modified version of the current `PATH` environment variable for
/// Node.js executables, which provides access to the Node shim but no other
/// Notion shims.
pub fn path_for_node_scripts() -> Fallible<OsString> {
    let prepended_paths = vec![path::bin_dir()?];
    build_path(prepended_paths)
}

/// Produces a modified version of the current `PATH` environment variable that
/// removes the Notion shims and binaries, to use for running system node and
/// executables.
pub fn path_for_system_node() -> Fallible<OsString> {
    let prepended_paths = vec![];
    build_path(prepended_paths)
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use std::env;
    use std::path::PathBuf;

    #[cfg(windows)]
    use winfolder;

    fn notion_base() -> PathBuf {
        #[cfg(unix)]
        return PathBuf::from(env::home_dir().expect("Could not get home directory")).join(".notion");

        #[cfg(all(windows, target_arch = "x86"))]
        return winfolder::Folder::ProgramFiles.path().join("Notion");

        #[cfg(all(windows, target_arch = "x86_64"))]
        return winfolder::Folder::ProgramFilesX64.path().join("Notion");
    }

    fn shim_dir() -> PathBuf {
        notion_base().join("shim")
    }
    fn bin_dir() -> PathBuf {
        notion_base().join("bin")
    }

    #[cfg(windows)]
    fn program_data_root() -> PathBuf {
        winfolder::Folder::ProgramData.path().join("Notion")
    }

    #[test]
    fn test_shell_name() {
        env::set_var("NOTION_SHELL", "bash");
        assert_eq!(shell_name().unwrap(), "bash".to_string());
    }

    #[test]
    fn test_postscript_path() {
        env::set_var("NOTION_POSTSCRIPT", "/some/path");
        assert_eq!(postscript_path().unwrap(), PathBuf::from("/some/path"));
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_installed_node() {
        env::set_var(
            "PATH",
            format!(
                "/usr/bin:{}:/blah:{}:/doesnt/matter/bin",
                bin_dir().to_string_lossy(),
                shim_dir().to_string_lossy()
            ),
        );

        let expected_node_bin = notion_base()
            .join("versions")
            .join("node")
            .join("1.2.3")
            .join("bin");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_node_bin.as_path().to_str().unwrap());
        expected_path.push_str(":/usr/bin:/blah:/doesnt/matter/bin");

        assert_eq!(
            path_for_installed_node("1.2.3").unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_installed_node() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(bin_dir());
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shim_bin = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim_bin);

        let expected_node_bin = program_data_root()
            .join("versions")
            .join("node")
            .join("1.2.3");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_node_bin.as_path().to_str().unwrap());
        expected_path.push_str(";C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(
            path_for_installed_node("1.2.3").unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_installed_yarn() {
        env::set_var(
            "PATH",
            format!(
                "{}:/usr/bin:/blah:/doesnt/matter/bin:{}",
                bin_dir().to_string_lossy(),
                shim_dir().to_string_lossy()
            ),
        );

        let expected_yarn_bin = notion_base()
            .join("versions")
            .join("yarn")
            .join("1.2.3")
            .join("bin");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_yarn_bin.as_path().to_str().unwrap());
        expected_path.push_str(":/usr/bin:/blah:/doesnt/matter/bin");

        assert_eq!(
            path_for_installed_yarn("1.2.3").unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_installed_yarn() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(bin_dir());
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\something"));
        pathbufs.push(PathBuf::from("D:\\\\blah"));

        let path_with_shim_bin = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim_bin);

        let expected_yarn_bin = program_data_root()
            .join("versions")
            .join("yarn")
            .join("1.2.3")
            .join("bin");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_yarn_bin.as_path().to_str().unwrap());
        expected_path.push_str(";C:\\\\something;D:\\\\blah");

        assert_eq!(
            path_for_installed_yarn("1.2.3").unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_node_scripts() {
        env::set_var(
            "PATH",
            format!(
                "{}:{}:/usr/bin:/blah:/doesnt/matter/bin",
                bin_dir().to_string_lossy(),
                shim_dir().to_string_lossy()
            ),
        );

        let mut expected_path = String::from("");
        expected_path.push_str(&bin_dir().to_string_lossy());
        expected_path.push_str(":/usr/bin:/blah:/doesnt/matter/bin");

        assert_eq!(
            path_for_node_scripts().unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_node_scripts() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(bin_dir());
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shim_bin = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim_bin);

        let expected_node_bin = program_data_root()
            .join("versions")
            .join("node")
            .join("1.2.3");

        let mut expected_path = String::from("");
        expected_path.push_str(&bin_dir().to_string_lossy());
        expected_path.push_str(";C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(
            path_for_node_scripts().unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_system_node() {
        env::set_var(
            "PATH",
            format!(
                "{}:/usr/bin:{}:/bin",
                bin_dir().to_string_lossy(),
                shim_dir().to_string_lossy()
            ),
        );

        let expected_path = String::from("/usr/bin:/bin");

        assert_eq!(path_for_system_node().unwrap().into_string().unwrap(), expected_path);
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_system_node() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(bin_dir());
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shim_bin = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim_bin);

        let expected_path = String::from("C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(path_for_system_node().unwrap().into_string().unwrap(), expected_path);
    }
}
