//! Provides utilities for modifying the environment when a shim calls out to
//! its delegated executable.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use path;

pub(crate) fn shell_name() -> Option<String> {
    env::var_os("NOTION_SHELL").map(|s| s.to_string_lossy().into_owned())
}

pub fn postscript_path() -> Option<PathBuf> {
    env::var_os("NOTION_POSTSCRIPT")
        .as_ref()
        .map(|ref s| Path::new(s).to_path_buf())
}

/// Produces a modified version of the current `PATH` environment variable that
/// will find Node.js executables in the installation directory for the given
/// version of Node instead of in the Notion shim directory.
pub fn path_for_installed_node(version: &str) -> OsString {
    let current = env::var_os("PATH").unwrap_or(OsString::new());
    let shim_dir = &path::shim_dir().unwrap();
    let split = env::split_paths(&current).filter(|s| s != shim_dir);
    let mut path_vec: Vec<PathBuf> = Vec::new();
    path_vec.push(path::node_version_bin_dir(version).unwrap());
    path_vec.push(path::yarn_version_bin_dir(version).unwrap());
    path_vec.extend(split);
    env::join_paths(path_vec.iter()).unwrap()
}

/// Produces a modified version of the current `PATH` environment variable that
/// removes the Notion shims and binaries, to use for running system node and
/// executables.
pub fn path_for_system_node() -> OsString {
    let current = env::var_os("PATH").unwrap_or(OsString::new());
    let shim_dir = &path::shim_dir().unwrap();
    // remove the shim dir from the path
    let split = env::split_paths(&current).filter(|s| s != shim_dir);
    env::join_paths(split).unwrap()
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use std::env;
    use std::path::PathBuf;

    #[cfg(windows)]
    use winfolder;

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
        let home = env::home_dir().expect("Could not get home directory");
        env::set_var("PATH", "/usr/bin:/blah:/doesnt/matter/bin");

        let mut expected_node_bin = PathBuf::from(&home);
        expected_node_bin.push(".notion");
        expected_node_bin.push("versions");
        expected_node_bin.push("node");
        expected_node_bin.push("1.2.3");
        expected_node_bin.push("bin");

        let mut expected_yarn_bin = PathBuf::from(&home);
        expected_yarn_bin.push(".notion");
        expected_yarn_bin.push("versions");
        expected_yarn_bin.push("yarn");
        expected_yarn_bin.push("1.2.3");
        expected_yarn_bin.push("bin");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_node_bin.as_path().to_str().unwrap());
        expected_path.push_str(":");
        expected_path.push_str(expected_yarn_bin.as_path().to_str().unwrap());
        expected_path.push_str(":/usr/bin:/blah:/doesnt/matter/bin");

        assert_eq!(
            path_for_installed_node("1.2.3").into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_installed_node() {
        let program_data = winfolder::Folder::ProgramData.path();
        env::set_var("PATH", "C:\\\\something;D:\\\\blah");

        let mut expected_node_bin = PathBuf::from(&program_data);
        expected_node_bin.push("Notion");
        expected_node_bin.push("versions");
        expected_node_bin.push("node");
        expected_node_bin.push("1.2.3");

        let mut expected_yarn_bin = PathBuf::from(&program_data);
        expected_yarn_bin.push("Notion");
        expected_yarn_bin.push("versions");
        expected_yarn_bin.push("yarn");
        expected_yarn_bin.push("1.2.3");
        expected_yarn_bin.push("bin");

        let mut expected_path = String::from("");
        expected_path.push_str(expected_node_bin.as_path().to_str().unwrap());
        expected_path.push_str(";");
        expected_path.push_str(expected_yarn_bin.as_path().to_str().unwrap());
        expected_path.push_str(";C:\\\\something;D:\\\\blah");

        assert_eq!(
            path_for_installed_node("1.2.3").into_string().unwrap(),
            expected_path
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_system_node() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();

        let home = env::home_dir().expect("Could not get home directory");
        let mut shim_dir = PathBuf::from(&home);
        shim_dir.push(".notion");
        shim_dir.push("bin");

        pathbufs.push(shim_dir);
        pathbufs.push(PathBuf::from("/usr/bin"));
        pathbufs.push(PathBuf::from("/bin"));

        let path_with_shim = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim);

        let expected_path = String::from("/usr/bin:/bin");

        assert_eq!(path_for_system_node().into_string().unwrap(), expected_path);
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_system_node() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();

        let program_files = if cfg!(target_arch = "x86_64") {
            winfolder::Folder::ProgramFilesX64.path()
        } else {
            winfolder::Folder::ProgramFiles.path()
        };
        let mut shim_dir = PathBuf::from(&program_files);
        shim_dir.push("Notion");
        shim_dir.push("bin");

        pathbufs.push(shim_dir);
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shim = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shim);

        let expected_path = String::from("C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(path_for_system_node().into_string().unwrap(), expected_path);
    }
}
