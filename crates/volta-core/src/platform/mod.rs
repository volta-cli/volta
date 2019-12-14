use std::env::JoinPathsError;
use std::ffi::OsString;
use std::path::PathBuf;

use envoy;
use semver::Version;

use crate::error::ErrorDetails;
use crate::layout::{env_paths, volta_home};
use crate::session::Session;
use crate::tool::load_default_npm_version;
use volta_fail::{Fallible, ResultExt};

pub mod sourced;
pub use self::sourced::{Source, SourcedImage, SourcedPlatformSpec};

/// A specification of tool versions needed for a platform
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlatformSpec {
    /// The pinned version of Node.
    pub node_runtime: Version,
    /// The pinned version of npm, if any.
    pub npm: Option<Version>,
    /// The pinned version of Yarn, if any.
    pub yarn: Option<Version>,
}

impl PlatformSpec {
    pub fn checkout(&self, session: &mut Session) -> Fallible<Image> {
        session.ensure_node(&self.node_runtime)?;

        if let Some(ref yarn_version) = self.yarn {
            session.ensure_yarn(yarn_version)?;
        }

        Ok(Image {
            node: self.node_runtime.clone(),
            npm: match self.npm {
                Some(ref version) => version.clone(),
                None => load_default_npm_version(&self.node_runtime)?,
            },
            yarn: self.yarn.clone(),
        })
    }
}

/// A platform image.
#[derive(Clone, Debug)]
pub struct Image {
    /// The pinned version of Node.
    pub node: Version,
    /// The pinned version of npm.
    pub npm: Version,
    /// The pinned version of Yarn, if any.
    pub yarn: Option<Version>,
}

impl Image {
    fn bins(&self) -> Fallible<Vec<PathBuf>> {
        let home = volta_home()?;
        let node_str = self.node.to_string();
        let npm_str = self.npm.to_string();
        // ISSUE(#292): Install npm, and handle using that
        let mut bins = vec![home.node_image_bin_dir(&node_str, &npm_str)];
        if let Some(ref yarn) = self.yarn {
            let yarn_str = yarn.to_string();
            bins.push(home.yarn_image_bin_dir(&yarn_str));
        }
        Ok(bins)
    }

    /// Produces a modified version of the current `PATH` environment variable that
    /// will find toolchain executables (Node, Yarn) in the installation directories
    /// for the given versions instead of in the Volta shim directory.
    pub fn path(&self) -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or_else(|| envoy::Var::from(""));
        let mut new_path = old_path.split();

        for remove_path in env_paths()? {
            new_path = new_path.remove(remove_path);
        }

        new_path
            .prefix(self.bins()?)
            .join()
            .with_context(build_path_error)
    }
}

/// A lightweight namespace type representing the system environment, i.e. the environment
/// with Volta removed.
pub struct System;

impl System {
    /// Produces a modified version of the current `PATH` environment variable that
    /// removes the Volta shims and binaries, to use for running system node and
    /// executables.
    pub fn path() -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or_else(|| envoy::Var::from(""));
        let mut new_path = old_path.split();

        for remove_path in env_paths()? {
            new_path = new_path.remove(remove_path);
        }

        new_path.join().with_context(build_path_error)
    }

    /// Reproduces the Volta-enabled `PATH` environment variable for situations where
    /// Volta has been deactivated
    #[cfg(not(feature = "volta-updates"))]
    pub fn enabled_path() -> Fallible<OsString> {
        let old_path = envoy::path().unwrap_or_else(|| envoy::Var::from(""));
        let mut new_path = old_path.split();

        for add_path in env_paths()? {
            if !old_path.split().any(|part| part == add_path) {
                new_path = new_path.prefix_entry(add_path);
            }
        }

        new_path.join().with_context(build_path_error)
    }
}

fn build_path_error(_err: &JoinPathsError) -> ErrorDetails {
    ErrorDetails::BuildPathError
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::layout::volta_home;
    #[cfg(windows)]
    use crate::layout::volta_install;
    use semver::Version;
    use std;
    #[cfg(not(feature = "volta-updates"))]
    use std::path::PathBuf;

    // Since unit tests are run in parallel, tests that modify the PATH environment variable are subject to race conditions
    // To prevent that, ensure that all tests that rely on PATH are run in serial by adding them to this meta-test
    #[test]
    fn test_paths() {
        test_image_path();
        test_system_path();
        #[cfg(not(feature = "volta-updates"))]
        test_system_enabled_path();
    }

    #[cfg(unix)]
    fn test_image_path() {
        std::env::set_var(
            "PATH",
            format!(
                "/usr/bin:/blah:{}:/doesnt/matter/bin",
                volta_home().unwrap().shim_dir().to_string_lossy()
            ),
        );

        let node_bin = volta_home()
            .unwrap()
            .root()
            .join("tools")
            .join("image")
            .join("node")
            .join("1.2.3")
            .join("6.4.3")
            .join("bin");
        let expected_node_bin = node_bin.as_path().to_str().unwrap();

        let yarn_bin = volta_home()
            .unwrap()
            .root()
            .join("tools")
            .join("image")
            .join("yarn")
            .join("4.5.7")
            .join("bin");
        let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

        let v123 = Version::parse("1.2.3").unwrap();
        let v457 = Version::parse("4.5.7").unwrap();
        let v643 = Version::parse("6.4.3").unwrap();

        let no_yarn_image = Image {
            node: v123.clone(),
            npm: v643.clone(),
            yarn: None,
        };

        assert_eq!(
            no_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin),
        );

        let with_yarn_image = Image {
            node: v123.clone(),
            npm: v643.clone(),
            yarn: Some(v457.clone()),
        };

        assert_eq!(
            with_yarn_image.path().unwrap().into_string().unwrap(),
            format!(
                "{}:{}:/usr/bin:/blah:/doesnt/matter/bin",
                expected_node_bin, expected_yarn_bin
            ),
        );
    }

    #[cfg(windows)]
    fn test_image_path() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        {
            #[cfg(feature = "volta-updates")]
            pathbufs.push(volta_install().unwrap().root().to_owned());
            #[cfg(not(feature = "volta-updates"))]
            pathbufs.push(volta_install().unwrap().bin_dir());
        }
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shims = std::env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        std::env::set_var("PATH", path_with_shims);

        let node_bin = volta_home()
            .unwrap()
            .root()
            .join("tools")
            .join("image")
            .join("node")
            .join("1.2.3")
            .join("6.4.3");
        let expected_node_bin = node_bin.as_path().to_str().unwrap();

        let yarn_bin = volta_home()
            .unwrap()
            .root()
            .join("tools")
            .join("image")
            .join("yarn")
            .join("4.5.7")
            .join("bin");
        let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

        let v123 = Version::parse("1.2.3").unwrap();
        let v457 = Version::parse("4.5.7").unwrap();
        let v643 = Version::parse("6.4.3").unwrap();

        let no_yarn_image = Image {
            node: v123.clone(),
            npm: v643.clone(),
            yarn: None,
        };

        assert_eq!(
            no_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{};C:\\\\somebin;D:\\\\ProbramFlies", expected_node_bin),
        );

        let with_yarn_image = Image {
            node: v123.clone(),
            npm: v643.clone(),
            yarn: Some(v457.clone()),
        };

        assert_eq!(
            with_yarn_image.path().unwrap().into_string().unwrap(),
            format!(
                "{};{};C:\\\\somebin;D:\\\\ProbramFlies",
                expected_node_bin, expected_yarn_bin
            ),
        );
    }

    #[cfg(unix)]
    fn test_system_path() {
        std::env::set_var(
            "PATH",
            format!(
                "{}:/usr/bin:/bin",
                volta_home().unwrap().shim_dir().to_string_lossy()
            ),
        );

        let expected_path = String::from("/usr/bin:/bin");

        assert_eq!(
            System::path().unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[cfg(windows)]
    fn test_system_path() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        {
            #[cfg(feature = "volta-updates")]
            pathbufs.push(volta_install().unwrap().root().to_owned());
            #[cfg(not(feature = "volta-updates"))]
            pathbufs.push(volta_install().unwrap().bin_dir());
        }
        pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

        let path_with_shims = std::env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        std::env::set_var("PATH", path_with_shims);

        let expected_path = String::from("C:\\\\somebin;D:\\\\ProbramFlies");

        assert_eq!(
            System::path().unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[cfg(all(unix, not(feature = "volta-updates")))]
    fn test_system_enabled_path() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
        pathbufs.push(PathBuf::from("/usr/bin"));
        pathbufs.push(PathBuf::from("/bin"));

        let expected_path = std::env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        // If the path already contains the shim dir, there shouldn't be any changes
        std::env::set_var("PATH", expected_path.clone());
        assert_eq!(
            System::enabled_path().unwrap().into_string().unwrap(),
            expected_path
        );

        // If the path doesn't contain the shim dir, it should be prefixed onto the existing path
        std::env::set_var("PATH", "/usr/bin:/bin");
        assert_eq!(
            System::enabled_path().unwrap().into_string().unwrap(),
            expected_path
        );
    }

    #[cfg(all(windows, not(feature = "volta-updates")))]
    fn test_system_enabled_path() {
        let mut pathbufs: Vec<PathBuf> = Vec::new();
        {
            #[cfg(feature = "volta-updates")]
            pathbufs.push(volta_install().unwrap().root().to_owned());
            #[cfg(not(feature = "volta-updates"))]
            pathbufs.push(volta_install().unwrap().bin_dir());
        }
        pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
        pathbufs.push(PathBuf::from("C:\\\\somebin"));
        pathbufs.push(PathBuf::from("D:\\\\Program Files"));

        let expected_path = std::env::join_paths(pathbufs.iter())
            .unwrap()
            .into_string()
            .expect("Could not create path containing shim dir");

        // If the path already contains the shim dir, there shouldn't be any changes
        std::env::set_var("PATH", expected_path.clone());
        assert_eq!(
            System::enabled_path().unwrap().into_string().unwrap(),
            expected_path
        );

        // If the path doesn't contain the shim dir, it should be prefixed onto the existing path
        std::env::set_var("PATH", "C:\\\\somebin;D:\\\\Program Files");
        assert_eq!(
            System::enabled_path().unwrap().into_string().unwrap(),
            expected_path
        );
    }
}
