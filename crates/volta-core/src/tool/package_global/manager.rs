use std::path::PathBuf;
use std::process::Command;

/// The package manager used to install a given package
#[derive(
    Copy, Clone, serde::Serialize, serde::Deserialize, PartialOrd, Ord, PartialEq, Eq, Debug,
)]
pub enum PackageManager {
    Npm,
    Yarn,
}

impl PackageManager {
    /// Given the `package_root`, returns the directory where the source is stored for this
    /// package manager. This will include the top-level `node_modules`, where appropriate.
    pub fn source_dir(self, package_root: PathBuf) -> PathBuf {
        let mut path = self.source_root(package_root);
        path.push("node_modules");

        path
    }

    /// Given the `package_root`, returns the root of the source directory. This directory will
    /// contain the top-level `node-modules`
    #[cfg(unix)]
    fn source_root(self, package_root: PathBuf) -> PathBuf {
        // On Unix, the source is always within a `lib` subdirectory, with both npm and Yarn
        let mut path = package_root;
        path.push("lib");

        path
    }

    /// Given the `package_root`, returns the root of the source directory. This directory will
    /// contain the top-level `node-modules`
    #[cfg(windows)]
    fn source_root(self, package_root: PathBuf) -> PathBuf {
        match self {
            // On Windows, npm puts the source node_modules directory in the root of the `prefix`
            PackageManager::Npm => package_root,
            // On Windows, we still tell yarn to use the `lib` subdirectory
            PackageManager::Yarn => {
                let mut path = package_root;
                path.push("lib");

                path
            }
        }
    }

    /// Given the `package_root`, returns the directory where binaries are stored for this package
    /// manager.
    #[cfg(unix)]
    pub fn binary_dir(self, package_root: PathBuf) -> PathBuf {
        // On Unix, the binaries are always within a `bin` subdirectory for both npm and Yarn
        let mut path = package_root;
        path.push("bin");

        path
    }

    /// Given the `package_root`, returns the directory where binaries are stored for this package
    /// manager.
    #[cfg(windows)]
    pub fn binary_dir(self, package_root: PathBuf) -> PathBuf {
        match self {
            // On Windows, npm leaves the binaries at the root of the `prefix` directory
            PackageManager::Npm => package_root,
            // On Windows, Yarn still includes the `bin` subdirectory
            PackageManager::Yarn => {
                let mut path = package_root;
                path.push("bin");

                path
            }
        }
    }

    /// Modify a given `Command` to be set up for global installs, given the package root
    pub(super) fn setup_global_command(self, command: &mut Command, package_root: PathBuf) {
        command.env("npm_config_prefix", &package_root);

        if let PackageManager::Yarn = self {
            command.env("npm_config_global_folder", self.source_root(package_root));
        }
    }
}
