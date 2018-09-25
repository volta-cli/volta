//! Provides utilities for modifying the environment when a shim calls out to
//! its delegated executable.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use image::Image;
use notion_fail::{Fallible, ResultExt};
use path;
use semver::Version;

pub(crate) fn shell_name() -> Option<String> {
    env::var_os("NOTION_SHELL").map(|s| s.to_string_lossy().into_owned())
}

pub fn postscript_path() -> Option<PathBuf> {
    env::var_os("NOTION_POSTSCRIPT")
        .as_ref()
        .map(|ref s| Path::new(s).to_path_buf())
}

// remove the Notion shims from the path, then prepend any input paths
fn build_path(paths: Vec<PathBuf>) -> Fallible<OsString> {
    let current_dir = env::var_os("PATH").unwrap_or(OsString::new());
    let shim_dir = &path::shim_dir()?;
    let split = env::split_paths(&current_dir).filter(|s| s != shim_dir);
    let mut path_vec: Vec<PathBuf> = Vec::new();
    for p in paths.iter() {
        path_vec.push(p.to_path_buf());
    }
    path_vec.extend(split);
    env::join_paths(path_vec.iter()).unknown()
}

/// Produces a modified version of the current `PATH` environment variable that
/// will find toolchain executables (Node, Yarn) in the installation directories
/// for the given versions instead of in the Notion shim directory.
pub fn path_for_platform(image: &Image) -> Fallible<OsString> {
    let mut prepended_paths = vec![path::node_version_bin_dir(&image.node_str)?];
    if let Some(ref version) = &image.yarn_str {
        prepended_paths.push(path::yarn_version_bin_dir(version)?);
    }
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
    use semver::Version;

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
    fn test_path_for_toolchain() {
        env::set_var(
            "PATH",
            format!(
                "/usr/bin:/blah:{}:/doesnt/matter/bin",
                shim_dir().to_string_lossy()
            ),
        );

        let node_bin = notion_base()
            .join("versions")
            .join("node")
            .join("1.2.3")
            .join("bin");
        let expected_node_bin = node_bin.as_path().to_str().unwrap();

        let yarn_bin = notion_base()
            .join("versions")
            .join("yarn")
            .join("4.5.7")
            .join("bin");
        let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

        assert_eq!(
            path_for_toolchain(&Version::parse("1.2.3").unwrap(), &None).unwrap().into_string().unwrap(),
            format!("{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin),
        );
        assert_eq!(
            path_for_toolchain(&Version::parse("1.2.3").unwrap(), &Version::parse("4.5.7").ok()).unwrap().into_string().unwrap(),
            format!("{}:{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin, expected_yarn_bin),
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_path_for_toolchain() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shims = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shims);

        let node_bin = program_data_root()
            .join("versions")
            .join("node")
            .join("1.2.3");
        let expected_node_bin = node_bin.as_path().to_str().unwrap();

        let yarn_bin = program_data_root()
            .join("versions")
            .join("yarn")
            .join("4.5.7")
            .join("bin");
        let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

        assert_eq!(
            path_for_toolchain(&Version::parse("1.2.3").unwrap(), &None).unwrap().into_string().unwrap(),
            format!("{};C:\\\\somebin;D:\\\\ProbramFlies", expected_node_bin),
        );
        assert_eq!(
            path_for_toolchain(&Version::parse("1.2.3").unwrap(), &Version::parse("4.5.7").ok()).unwrap().into_string().unwrap(),
            format!("{};{};C:\\\\somebin;D:\\\\ProbramFlies", expected_node_bin, expected_yarn_bin),
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_path_for_system_node() {
        env::set_var(
            "PATH",
            format!(
                "{}:/usr/bin:/bin",
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
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shims = env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        env::set_var("PATH", path_with_shims);

        let expected_path = String::from("C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(path_for_system_node().unwrap().into_string().unwrap(), expected_path);
    }
}
