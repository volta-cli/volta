use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json;

use volta_core::distro::node::NodeDistro;
use volta_core::distro::yarn::YarnDistro;
use volta_core::fs;
use volta_core::layout::layout;

use test_support::{self, ok_or_panic, paths, paths::PathExt, process::ProcessBuilder};

#[derive(PartialEq, Clone)]
pub struct FileBuilder {
    path: PathBuf,
    contents: String,
}

impl FileBuilder {
    pub fn new(path: PathBuf, contents: &str) -> FileBuilder {
        FileBuilder {
            path,
            contents: contents.to_string(),
        }
    }

    pub fn build(&self) {
        self.dirname().mkdir_p();

        let mut file = File::create(&self.path)
            .unwrap_or_else(|e| panic!("could not create file {}: {}", self.path.display(), e));

        ok_or_panic! { file.write_all(self.contents.as_bytes()) };
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

#[must_use]
pub struct TempProjectBuilder {
    root: TempProject,
    files: Vec<FileBuilder>,
}

impl TempProjectBuilder {
    /// Root of the project, ex: `/path/to/cargo/target/smoke_test/t0/foo`
    pub fn root(&self) -> PathBuf {
        self.root.root()
    }

    pub fn new(root: PathBuf) -> TempProjectBuilder {
        TempProjectBuilder {
            root: TempProject { root },
            files: vec![],
        }
    }

    /// Set the package.json for the temporary project (chainable)
    pub fn package_json(mut self, contents: &str) -> Self {
        let package_file = package_json_file(self.root());
        self.files.push(FileBuilder::new(package_file, contents));
        self
    }

    /// Create the project
    pub fn build(self) -> TempProject {
        // First, clean the temporary project directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        let layout = ok_or_panic!(layout());

        // make sure these directories exist and are empty
        layout.user.node_cache_dir().ensure_empty();
        layout.user.shim_dir().ensure_empty();
        layout.user.node_inventory_dir().ensure_empty();
        layout.user.yarn_inventory_dir().ensure_empty();
        layout.user.package_inventory_dir().ensure_empty();
        layout.user.node_image_root_dir().ensure_empty();
        layout.user.yarn_image_root_dir().ensure_empty();
        layout.user.package_image_root_dir().ensure_empty();
        layout.user.user_toolchain_dir().ensure_empty();
        layout.user.tmp_dir().ensure_empty();
        // and these files do not exist
        layout.install.notion_file().rm();
        layout.install.shim_executable().rm();
        layout.user.user_hooks_file().rm();
        layout.user.user_platform_file().rm();
        // create symlinks to shim executable for node, yarn, and packages
        ok_or_panic!(fs::symlink_file(shim_exe(), self.root.node_exe()));
        ok_or_panic!(fs::symlink_file(shim_exe(), self.root.yarn_exe()));
        ok_or_panic!(fs::symlink_file(
            shim_exe(),
            layout.install.shim_executable()
        ));

        // write files
        for file_builder in self.files {
            file_builder.build();
        }

        let TempProjectBuilder { root, .. } = self;
        root
    }

    fn rm_root(&self) {
        self.root.root().rm_rf()
    }
}

// files and dirs in the temporary project

fn package_json_file(mut root: PathBuf) -> PathBuf {
    root.push("package.json");
    root
}

pub struct TempProject {
    root: PathBuf,
}

impl TempProject {
    /// Root of the project, ex: `/path/to/cargo/target/integration_test/t0/foo`
    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    /// Create a `ProcessBuilder` to run a program in the project.
    /// Example:
    ///         assert_that(
    ///             p.process(&p.bin("foo")),
    ///             execs().with_stdout("bar\n"),
    ///         );
    pub fn process<T: AsRef<OsStr>>(&self, program: T) -> ProcessBuilder {
        let mut p = test_support::process::process(program);
        p.cwd(self.root())
            // setup the Volta environment
            .env_remove("VOLTA_NODE_VERSION")
            .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows

        p
    }

    /// Create a `ProcessBuilder` to run volta.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.volta("use node 9.5"), execs());
    pub fn volta(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&volta_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Create a `ProcessBuilder` to run Node.
    pub fn node(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.node_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn node_exe(&self) -> PathBuf {
        self.root().join(format!("node{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Yarn.
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.yarn_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn yarn_exe(&self) -> PathBuf {
        self.root().join(format!("yarn{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Npm.
    pub fn npm(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.npm_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn npm_exe(&self) -> PathBuf {
        self.root().join(format!("npm{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run a package executable.
    pub fn exec_shim(&self, exe: &str, cmd: &str) -> ProcessBuilder {
        let shim_file = ok_or_panic!(layout()).user.shim_file(exe);
        let mut p = self.process(shim_file);
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Verify that the input Node version has been fetched.
    pub fn node_version_is_fetched(&self, version: &str) -> bool {
        let layout = ok_or_panic!(layout());
        let distro_file_name = NodeDistro::filename(version);
        let inventory_dir = layout.user.node_inventory_dir();
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Node version has been unpacked.
    pub fn node_version_is_unpacked(&self, version: &str, npm_version: &str) -> bool {
        let unpack_dir = ok_or_panic!(layout()).user.node_image_bin_dir(version, npm_version);
        unpack_dir.exists()
    }

    /// Verify that the input Node version has been installed.
    pub fn assert_node_version_is_installed(&self, version: &str, npm_version: &str) -> () {
        let layout = ok_or_panic!(layout());
        let user_platform = layout.user.user_platform_file();
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["runtime"], version);
        assert_eq!(json_contents["node"]["npm"], npm_version);
    }

    /// Verify that the input Yarn version has been fetched.
    pub fn yarn_version_is_fetched(&self, version: &str) -> bool {
        let layout = ok_or_panic!(layout());
        let distro_file_name = YarnDistro::filename(version);
        let inventory_dir = layout.user.yarn_inventory_dir();
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Yarn version has been unpacked.
    pub fn yarn_version_is_unpacked(&self, version: &str) -> bool {
        let unpack_dir = ok_or_panic!(layout()).user.yarn_image_dir(version);
        unpack_dir.exists()
    }

    /// Verify that the input Yarn version has been installed.
    pub fn assert_yarn_version_is_installed(&self, version: &str) -> () {
        let layout = ok_or_panic!(layout());
        let user_platform = layout.user.user_platform_file();
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["yarn"], version);
    }

    /// Verify that the input Npm version has been fetched.
    pub fn npm_version_is_fetched(&self, version: &str) -> bool {
        let layout = ok_or_panic!(layout());
        // ISSUE(#292): This is maybe the wrong place to put npm?
        let package_file = layout.user.package_distro_file("npm", version);
        let shasum_file = layout.user.package_distro_shasum("npm", version);
        package_file.exists() && shasum_file.exists()
    }

    /// Verify that the input Npm version has been unpacked.
    pub fn npm_version_is_unpacked(&self, version: &str) -> bool {
        // ISSUE(#292): This is maybe the wrong place to unpack npm?
        let unpack_dir = ok_or_panic!(layout()).user.package_image_dir("npm", version);
        unpack_dir.exists()
    }

    /// Verify that the input Npm version has been installed.
    pub fn assert_npm_version_is_installed(&self, version: &str) -> () {
        let layout = ok_or_panic!(layout());
        let user_platform = layout.user.user_platform_file();
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["npm"], version);
    }

    /// Verify that the input package version has been fetched.
    pub fn package_version_is_fetched(&self, name: &str, version: &str) -> bool {
        let layout = ok_or_panic!(layout());
        let package_file = layout.user.package_distro_file(name, version);
        let shasum_file = layout.user.package_distro_shasum(name, version);
        package_file.exists() && shasum_file.exists()
    }

    /// Verify that the input package version has been unpacked.
    pub fn package_version_is_unpacked(&self, name: &str, version: &str) -> bool {
        let unpack_dir = ok_or_panic!(layout()).user.package_image_dir(name, version);
        unpack_dir.exists()
    }

    /// Verify that the input package version has been fetched.
    pub fn shim_exists(&self, name: &str) -> bool {
        let shim_file = ok_or_panic!(layout()).user.shim_file(name);
        shim_file.exists()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        self.root().rm_rf();
    }
}

// Generates a temporary project environment
pub fn temp_project() -> TempProjectBuilder {
    TempProjectBuilder::new(paths::root().join("temp-project"))
}

// Path to compiled executables
pub fn cargo_dir() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
}

fn volta_exe() -> PathBuf {
    cargo_dir().join(format!("volta{}", env::consts::EXE_SUFFIX))
}

fn shim_exe() -> PathBuf {
    cargo_dir().join(format!("shim{}", env::consts::EXE_SUFFIX))
}

fn split_and_add_args(p: &mut ProcessBuilder, s: &str) {
    for arg in s.split_whitespace() {
        if arg.contains('"') || arg.contains('\'') {
            panic!("shell-style argument parsing is not supported")
        }
        p.arg(arg);
    }
}

fn read_file_to_string(file_path: impl AsRef<Path>) -> String {
    let mut contents = String::new();
    let mut file = ok_or_panic! { File::open(file_path) };
    ok_or_panic! { file.read_to_string(&mut contents) };
    contents
}
