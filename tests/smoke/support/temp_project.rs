use node_semver::Version;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use volta_core::fs::symlink_file;
use volta_core::tool::Node;

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

pub struct EnvVar {
    name: String,
    value: String,
}

impl EnvVar {
    pub fn new(name: &str, value: &str) -> Self {
        EnvVar {
            name: name.to_string(),
            value: value.to_string(),
        }
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
            root: TempProject {
                root: root.clone(),
                path: OsString::new(),
                env_vars: vec![],
            },
            files: vec![],
        }
    }

    /// Set the package.json for the temporary project (chainable)
    pub fn package_json(mut self, contents: &str) -> Self {
        let package_file = package_json_file(self.root());
        self.files.push(FileBuilder::new(package_file, contents));
        self
    }

    /// Create a file in the project directory (chainable)
    pub fn project_file(mut self, path: &str, contents: &str) -> Self {
        let path = self.root().join(path);
        self.files.push(FileBuilder::new(path, contents));
        self
    }

    /// Create a file in the `volta_home` directory (chainable)
    pub fn volta_home_file(mut self, path: &str, contents: &str) -> Self {
        let path = volta_home(self.root()).join(path);
        self.files.push(FileBuilder::new(path, contents));
        self
    }

    /// Set an environment variable (chainable)
    pub fn env(mut self, name: &str, value: &str) -> Self {
        self.root.env_vars.push(EnvVar::new(name, value));
        self
    }

    /// Create the project
    pub fn build(mut self) -> TempProject {
        // First, clean the temporary project directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        // make sure these directories exist and are empty
        node_cache_dir(self.root()).ensure_empty();
        volta_bin_dir(self.root()).ensure_empty();
        node_inventory_dir(self.root()).ensure_empty();
        yarn_inventory_dir(self.root()).ensure_empty();
        package_inventory_dir(self.root()).ensure_empty();
        node_image_root_dir(self.root()).ensure_empty();
        yarn_image_root_dir(self.root()).ensure_empty();
        package_image_root_dir(self.root()).ensure_empty();
        default_toolchain_dir(self.root()).ensure_empty();
        volta_tmp_dir(self.root()).ensure_empty();

        // and these files do not exist
        volta_file(self.root()).rm();
        shim_executable(self.root()).rm();
        default_hooks_file(self.root()).rm();
        default_platform_file(self.root()).rm();

        // create symlinks to shim executable for node, yarn, npm, and packages
        ok_or_panic!(symlink_file(shim_exe(), self.root.node_exe()));
        ok_or_panic!(symlink_file(shim_exe(), self.root.yarn_exe()));
        ok_or_panic!(symlink_file(shim_exe(), self.root.npm_exe()));

        // write files
        for file_builder in self.files {
            file_builder.build();
        }

        // prepend Volta bin dir to the PATH
        let current_path = envoy::path().expect("Could not get current PATH");
        let new_path = current_path.split();
        self.root.path = new_path
            .prefix_entry(volta_bin_dir(self.root.root()))
            .join()
            .expect("Failed to join paths");

        let TempProjectBuilder { root, .. } = self;
        root
    }

    fn rm_root(&self) {
        self.root.root().rm_rf()
    }
}

// files and dirs in the temporary project

fn home_dir(root: PathBuf) -> PathBuf {
    root.join("home")
}
fn volta_home(root: PathBuf) -> PathBuf {
    home_dir(root).join(".volta")
}
fn volta_file(root: PathBuf) -> PathBuf {
    volta_home(root).join("volta")
}
fn shim_executable(root: PathBuf) -> PathBuf {
    volta_bin_dir(root).join("volta-shim")
}
fn default_hooks_file(root: PathBuf) -> PathBuf {
    volta_home(root).join("hooks.json")
}
fn volta_tmp_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("tmp")
}
fn volta_bin_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("bin")
}
fn volta_tools_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("tools")
}
fn inventory_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("inventory")
}
fn default_toolchain_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("user")
}
fn image_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("image")
}
fn node_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("node")
}
fn node_image_dir(node: &str, root: PathBuf) -> PathBuf {
    node_image_root_dir(root).join(node)
}
fn node_image_bin_dir(node: &str, root: PathBuf) -> PathBuf {
    node_image_dir(node, root).join("bin")
}
fn npm_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("npm")
}
fn npm_image_dir(version: &str, root: PathBuf) -> PathBuf {
    npm_image_root_dir(root).join(version)
}
fn npm_image_bin_dir(version: &str, root: PathBuf) -> PathBuf {
    npm_image_dir(version, root).join("bin")
}
fn yarn_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("yarn")
}
fn yarn_image_dir(version: &str, root: PathBuf) -> PathBuf {
    yarn_image_root_dir(root).join(version)
}
fn package_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("packages")
}
fn node_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("node")
}
fn npm_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("npm")
}
fn yarn_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("yarn")
}
fn package_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("packages")
}
fn cache_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("cache")
}
fn node_cache_dir(root: PathBuf) -> PathBuf {
    cache_dir(root).join("node")
}
fn package_json_file(mut root: PathBuf) -> PathBuf {
    root.push("package.json");
    root
}
fn shim_file(name: &str, root: PathBuf) -> PathBuf {
    volta_bin_dir(root).join(format!("{}{}", name, env::consts::EXE_SUFFIX))
}
fn package_image_dir(name: &str, root: PathBuf) -> PathBuf {
    image_dir(root).join("packages").join(name)
}
fn default_platform_file(root: PathBuf) -> PathBuf {
    default_toolchain_dir(root).join("platform.json")
}
pub fn node_distro_file_name(version: &str) -> String {
    let version = Version::parse(version).unwrap();
    Node::archive_filename(&version)
}
fn npm_distro_file_name(version: &str) -> String {
    package_distro_file_name("npm", version)
}
fn yarn_distro_file_name(version: &str) -> String {
    format!("yarn-v{}.tar.gz", version)
}
fn package_distro_file_name(name: &str, version: &str) -> String {
    format!("{}-{}.tgz", name, version)
}

pub struct TempProject {
    root: PathBuf,
    path: OsString,
    env_vars: Vec<EnvVar>,
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
            .env("PATH", &self.path)
            .env("HOME", home_dir(self.root()))
            .env("VOLTA_HOME", volta_home(self.root()))
            .env("VOLTA_INSTALL_DIR", cargo_dir())
            .env_remove("VOLTA_NODE_VERSION")
            .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows

        // overrides for env vars
        for env_var in &self.env_vars {
            p.env(&env_var.name, &env_var.value);
        }

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
        volta_bin_dir(self.root()).join(format!("node{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Yarn.
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.yarn_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn yarn_exe(&self) -> PathBuf {
        volta_bin_dir(self.root()).join(format!("yarn{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Npm.
    pub fn npm(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.npm_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn npm_exe(&self) -> PathBuf {
        volta_bin_dir(self.root()).join(format!("npm{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run a package executable.
    pub fn exec_shim(&self, exe: &str, cmd: &str) -> ProcessBuilder {
        let shim_file = shim_file(exe, self.root());
        let mut p = self.process(shim_file);
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Verify that the input Node version has been fetched.
    pub fn node_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = node_distro_file_name(version);
        let inventory_dir = node_inventory_dir(self.root());
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Node version has been unpacked.
    pub fn node_version_is_unpacked(&self, version: &str) -> bool {
        let unpack_dir = node_image_bin_dir(version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input Node version has been installed.
    pub fn assert_node_version_is_installed(&self, version: &str) -> () {
        let default_platform = default_platform_file(self.root());
        let platform_contents = read_file_to_string(default_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["runtime"], version);
    }

    /// Verify that the input Yarn version has been fetched.
    pub fn yarn_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = yarn_distro_file_name(version);
        let inventory_dir = yarn_inventory_dir(self.root());
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Yarn version has been unpacked.
    pub fn yarn_version_is_unpacked(&self, version: &str) -> bool {
        let unpack_dir = yarn_image_dir(version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input Yarn version has been installed.
    pub fn assert_yarn_version_is_installed(&self, version: &str) -> () {
        let default_platform = default_platform_file(self.root());
        let platform_contents = read_file_to_string(default_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["yarn"], version);
    }

    /// Verify that the input Npm version has been fetched.
    pub fn npm_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = npm_distro_file_name(version);
        let inventory_dir = npm_inventory_dir(self.root());
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Npm version has been unpacked.
    pub fn npm_version_is_unpacked(&self, version: &str) -> bool {
        npm_image_bin_dir(version, self.root()).exists()
    }

    /// Verify that the input Npm version has been installed.
    pub fn assert_npm_version_is_installed(&self, version: &str) -> () {
        let default_platform = default_platform_file(self.root());
        let platform_contents = read_file_to_string(default_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["npm"], version);
    }

    /// Verify that the input package has been installed
    pub fn package_is_installed(&self, name: &str) -> bool {
        let install_dir = package_image_dir(name, self.root());
        install_dir.exists()
    }

    /// Verify that the input package version has been fetched.
    pub fn shim_exists(&self, name: &str) -> bool {
        shim_file(name, self.root()).exists()
    }

    /// Verify that a given path in the project directory exists
    pub fn project_path_exists(&self, path: &str) -> bool {
        self.root().join(path).exists()
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
    cargo_dir().join(format!("volta-shim{}", env::consts::EXE_SUFFIX))
}

fn split_and_add_args(p: &mut ProcessBuilder, s: &str) {
    for arg in s.split_whitespace() {
        if arg.contains('"') || arg.contains('\'') {
            panic!("shell-style argument parsing is not supported")
        }
        p.arg(arg);
    }
}

fn read_file_to_string(file_path: PathBuf) -> String {
    let mut contents = String::new();
    let mut file = ok_or_panic! { File::open(file_path) };
    ok_or_panic! { file.read_to_string(&mut contents) };
    contents
}
