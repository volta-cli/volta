use std::env::JoinPathsError;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use envoy;
use semver::Version;

use crate::error::ErrorDetails;
use crate::layout::{env_paths, volta_home};
use crate::session::Session;
use crate::tool::{load_default_npm_version, Node, Yarn};
use volta_fail::{Fallible, ResultExt};

/// The source with which a version is associated
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// Represents a version from the user default platform
    Default,

    /// Represents a version from a project manifest
    Project,

    /// Represents a version from a pinned Binary platform
    Binary,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Default => write!(f, "default"),
            Source::Project => write!(f, "project"),
            Source::Binary => write!(f, "binary"),
        }
    }
}

/// A version tagged with its source
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcedVersion {
    pub version: Version,
    pub source: Source,
}

impl SourcedVersion {
    pub fn default(version: Version) -> Self {
        SourcedVersion {
            version,
            source: Source::Default,
        }
    }

    pub fn project(version: Version) -> Self {
        SourcedVersion {
            version,
            source: Source::Project,
        }
    }

    pub fn binary(version: Version) -> Self {
        SourcedVersion {
            version,
            source: Source::Binary,
        }
    }
}

/// A specification of tool versions needed for a platform
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlatformSpec {
    /// The pinned version of Node
    pub node: SourcedVersion,
    /// The pinned version of npm, if any
    pub npm: Option<SourcedVersion>,
    /// The pinned version of Yarn, if any
    pub yarn: Option<SourcedVersion>,
}

impl PlatformSpec {
    pub fn checkout(&self, session: &mut Session) -> Fallible<Image> {
        ensure_node(&self.node.version, session)?;

        if let Some(yarn) = &self.yarn {
            ensure_yarn(&yarn.version, session)?;
        }

        Ok(Image {
            node: self.node.clone(),
            npm: match &self.npm {
                Some(npm) => npm.clone(),
                None => SourcedVersion {
                    version: load_default_npm_version(&self.node.version)?,
                    source: self.node.source,
                },
            },
            yarn: self.yarn.clone(),
        })
    }

    pub fn merge(&self, other: &Self) -> Self {
        PlatformSpec {
            node: self.node.clone(),
            npm: self.npm.as_ref().cloned().or_else(|| other.npm.clone()),
            yarn: self.yarn.as_ref().cloned().or_else(|| other.yarn.clone()),
        }
    }
}

/// Ensures that a specific Node version has been fetched and unpacked
fn ensure_node(version: &Version, session: &mut Session) -> Fallible<()> {
    let inventory = session.inventory()?;

    if !inventory.node.versions.contains(version) {
        Node::new(version.clone()).fetch_internal(session)?;
    }

    Ok(())
}

/// Ensures that a specific Yarn version has been fetched and unpacked
fn ensure_yarn(version: &Version, session: &mut Session) -> Fallible<()> {
    let inventory = session.inventory()?;

    if !inventory.yarn.versions.contains(version) {
        Yarn::new(version.clone()).fetch_internal(session)?;
    }

    Ok(())
}

/// A platform image
pub struct Image {
    /// The selected version of Node
    pub node: SourcedVersion,
    /// The resolved version of npm (either bundled or custom)
    pub npm: SourcedVersion,
    /// The selected version of Yarn, if any
    pub yarn: Option<SourcedVersion>,
}

impl Image {
    fn bins(&self) -> Fallible<Vec<PathBuf>> {
        let home = volta_home()?;
        let node_str = self.node.version.to_string();
        let npm_str = self.npm.version.to_string();
        // ISSUE(#292): Install npm, and handle using that
        let mut bins = vec![home.node_image_bin_dir(&node_str, &npm_str)];
        if let Some(yarn) = &self.yarn {
            let yarn_str = yarn.version.to_string();
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
            node: SourcedVersion::default(v123.clone()),
            npm: SourcedVersion::default(v643.clone()),
            yarn: None,
        };

        assert_eq!(
            no_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin),
        );

        let with_yarn_image = Image {
            node: SourcedVersion::default(v123.clone()),
            npm: SourcedVersion::default(v643.clone()),
            yarn: Some(SourcedVersion::default(v457.clone())),
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
            node: SourcedVersion::default(v123.clone()),
            npm: SourcedVersion::default(v643.clone()),
            yarn: None,
        };

        assert_eq!(
            no_yarn_image.path().unwrap().into_string().unwrap(),
            format!("{};C:\\\\somebin;D:\\\\ProbramFlies", expected_node_bin),
        );

        let with_yarn_image = Image {
            node: SourcedVersion::default(v123.clone()),
            npm: SourcedVersion::default(v643.clone()),
            yarn: Some(SourcedVersion::default(v457.clone())),
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
