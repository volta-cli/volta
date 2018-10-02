use std::ffi::OsString;
use std::path::PathBuf;

use envoy;
use semver::Version;

use notion_fail::{Fallible, ResultExt};
use path;

/// A toolchain manifest.
pub struct Image {
    /// The pinned version of Node, under the `toolchain.node` key.
    pub node: Version,
    /// The pinned version of Node as a string.
    pub node_str: String,
    /// The pinned version of Yarn, under the `toolchain.yarn` key.
    pub yarn: Option<Version>,
    /// The pinned version of Yarn as a string.
    pub yarn_str: Option<String>,
}

impl Image {
    pub fn bins(&self) -> Fallible<Vec<PathBuf>> {
        let mut bins = vec![path::node_version_bin_dir(&self.node_str)?];
        if let Some(ref yarn_str) = self.yarn_str {
            bins.push(path::yarn_version_bin_dir(yarn_str)?);
        }
        Ok(bins)
    }

    /// Produces a modified version of the current `PATH` environment variable that
    /// will find toolchain executables (Node, Yarn) in the installation directories
    /// for the given versions instead of in the Notion shim directory.
    pub fn path(&self) -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or(envoy::Var::from(""));

        let new_path = old_path
            .split()
            .remove(path::shim_dir()?)
            .prefix(self.bins()?)
            .join()
            .unknown()?;

        Ok(new_path)
    }
}

/// A lightweight namespace type representing the system environment, i.e. the environment
/// with Notion removed.
pub struct System;

impl System {

    /// Produces a modified version of the current `PATH` environment variable that
    /// removes the Notion shims and binaries, to use for running system node and
    /// executables.
    pub fn path() -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or(envoy::Var::from(""));

        let new_path = old_path
            .split()
            .remove(path::shim_dir()?)
            .join()
            .unknown()?;

        Ok(new_path)
    }

}

#[cfg(test)]
mod test {

    use super::*;
    use std;
    use std::path::PathBuf;
    use semver::Version;

    #[cfg(windows)]
    use winfolder;

    fn notion_base() -> PathBuf {
        #[cfg(unix)]
        return PathBuf::from(std::env::home_dir().expect("Could not get home directory")).join(".notion");

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
    #[cfg(unix)]
    fn test_image_path() {
        std::env::set_var(
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

        let v123 = Version::parse("1.2.3").unwrap();
        let v457 = Version::parse("4.5.7").unwrap();

        let no_yarn_image = Image {
            node: v123.clone(),
            node_str: v123.to_string(),
            yarn: None,
            yarn_str: None
        };

        assert_eq!(
            no_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin),
        );

        let with_yarn_image = Image {
            node: v123.clone(),
            node_str: v123.to_string(),
            yarn: Some(v457.clone()),
            yarn_str: Some(v457.to_string())
        };

        assert_eq!(
            with_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{}:{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin, expected_yarn_bin),
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_image_path() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(shim_dir());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shims = std::env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        std::env::set_var("PATH", path_with_shims);

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
    fn test_system_path() {
        std::env::set_var(
            "PATH",
            format!(
                "{}:/usr/bin:/bin",
                shim_dir().to_string_lossy()
            ),
        );

        let expected_path = String::from("/usr/bin:/bin");

        assert_eq!(System::path().unwrap().into_string().unwrap(), expected_path);
    }

    #[test]
    #[cfg(windows)]
    fn test_system_path() {
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