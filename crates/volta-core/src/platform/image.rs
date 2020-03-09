use std::ffi::OsString;
use std::path::PathBuf;

use super::{build_path_error, Sourced};
use crate::layout::{env_paths, volta_home};
use semver::Version;
use volta_fail::{Fallible, ResultExt};

/// A platform image.
pub struct Image {
    /// The pinned version of Node.
    pub node: Sourced<Version>,
    /// The resolved version of npm.
    pub npm: Sourced<Version>,
    /// The pinned version of Yarn, if any.
    pub yarn: Option<Sourced<Version>>,
}

impl Image {
    fn bins(&self) -> Fallible<Vec<PathBuf>> {
        let home = volta_home()?;
        let node_str = self.node.value.to_string();
        let npm_str = self.npm.value.to_string();
        // ISSUE(#292): Install npm, and handle using that
        let mut bins = vec![home.node_image_bin_dir(&node_str, &npm_str)];
        if let Some(yarn) = &self.yarn {
            let yarn_str = yarn.value.to_string();
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
