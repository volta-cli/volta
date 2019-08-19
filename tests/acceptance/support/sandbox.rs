use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use reqwest::hyper_011::header::HttpDate;

use test_support::{self, ok_or_panic, paths, paths::PathExt, process::ProcessBuilder};

use volta_core::path::{
    archive_extension, create_file_symlink, node_distro_file_name, yarn_distro_file_name, ARCH, OS,
};

#[cfg(feature = "mock-network")]
use mockito::{self, mock, Matcher};

// version cache for node and yarn
#[derive(PartialEq, Clone)]
struct CacheBuilder {
    path: PathBuf,
    expiry_path: PathBuf,
    contents: String,
    expired: bool,
}

impl CacheBuilder {
    #[allow(dead_code)]
    pub fn new(path: PathBuf, expiry_path: PathBuf, contents: &str, expired: bool) -> CacheBuilder {
        CacheBuilder {
            path,
            expiry_path,
            contents: contents.to_string(),
            expired,
        }
    }

    fn build(&self) {
        self.dirname().mkdir_p();

        // write cache file
        let mut cache_file = File::create(&self.path).unwrap_or_else(|e| {
            panic!("could not create cache file {}: {}", self.path.display(), e)
        });
        ok_or_panic! { cache_file.write_all(self.contents.as_bytes()) };

        // write expiry file
        let one_day = Duration::from_secs(24 * 60 * 60);
        let expiry_date = HttpDate::from(if self.expired {
            SystemTime::now() - one_day
        } else {
            SystemTime::now() + one_day
        });
        let mut expiry_file = File::create(&self.expiry_path).unwrap_or_else(|e| {
            panic!(
                "could not create cache expiry file {}: {}",
                self.expiry_path.display(),
                e
            )
        });
        ok_or_panic! { expiry_file.write_all(expiry_date.to_string().as_bytes()) };
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

// environment variables
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

// used to construct sandboxed package.json and platform.json
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
pub struct SandboxBuilder {
    root: Sandbox,
    files: Vec<FileBuilder>,
    caches: Vec<CacheBuilder>,
    path_dirs: Vec<PathBuf>,
}

pub trait DistroFixture: From<DistroMetadata> {
    fn server_path(&self) -> String;
    fn fixture_path(&self) -> String;
    fn metadata(&self) -> &DistroMetadata;
}

#[derive(Clone)]
pub struct DistroMetadata {
    pub version: &'static str,
    pub compressed_size: u32,
    pub uncompressed_size: Option<u32>,
}

pub struct NodeFixture {
    pub metadata: DistroMetadata,
}

pub struct YarnFixture {
    pub metadata: DistroMetadata,
}

impl From<DistroMetadata> for NodeFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl From<DistroMetadata> for YarnFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl DistroFixture for NodeFixture {
    fn server_path(&self) -> String {
        let version = &self.metadata.version;
        let extension = archive_extension();
        format!(
            "/v{}/node-v{}-{}-{}.{}",
            version, version, OS, ARCH, extension
        )
    }

    fn fixture_path(&self) -> String {
        let version = &self.metadata.version;
        let extension = archive_extension();
        format!(
            "tests/fixtures/node-v{}-{}-{}.{}",
            version, OS, ARCH, extension
        )
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl DistroFixture for YarnFixture {
    fn server_path(&self) -> String {
        let version = &self.metadata.version;
        format!("/v{}/yarn-v{}.tar.gz", version, version)
    }

    fn fixture_path(&self) -> String {
        format!("tests/fixtures/yarn-v{}.tar.gz", self.metadata.version)
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl SandboxBuilder {
    /// Root of the project, ex: `/path/to/cargo/target/integration_test/t0/foo`
    pub fn root(&self) -> PathBuf {
        self.root.root()
    }

    pub fn new(root: PathBuf) -> SandboxBuilder {
        SandboxBuilder {
            root: Sandbox {
                root,
                mocks: vec![],
                env_vars: vec![],
                env_vars_remove: vec![],
                path: OsString::new(),
            },
            files: vec![],
            caches: vec![],
            path_dirs: vec![volta_bin_dir()],
        }
    }

    #[allow(dead_code)]
    /// Set the Node cache for the sandbox (chainable)
    pub fn node_cache(mut self, cache: &str, expired: bool) -> Self {
        self.caches.push(CacheBuilder::new(
            node_index_file(),
            node_index_expiry_file(),
            cache,
            expired,
        ));
        self
    }

    /// Set the package.json for the sandbox (chainable)
    pub fn package_json(mut self, contents: &str) -> Self {
        let package_file = package_json_file(self.root());
        self.files.push(FileBuilder::new(package_file, contents));
        self
    }

    /// Set the platform.json for the sandbox (chainable)
    pub fn platform(mut self, contents: &str) -> Self {
        self.files
            .push(FileBuilder::new(user_platform_file(), contents));
        self
    }

    /// Set the shell for the sandbox (chainable)
    pub fn volta_shell(self, shell_name: &str) -> Self {
        self.env("VOLTA_SHELL", shell_name)
    }

    /// Set an environment variable for the sandbox (chainable)
    pub fn env(mut self, name: &str, value: &str) -> Self {
        self.root.env_vars.push(EnvVar::new(name, value));
        self
    }

    /// Add a directory to the PATH (chainable)
    pub fn path_dir(mut self, dir: &str) -> Self {
        self.path_dirs.push(PathBuf::from(dir));
        self
    }

    /// Setup mock to return the available node versions (chainable)
    pub fn node_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/node-dist/index.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);

        self
    }

    /// Setup mock to return the available yarn versions (chainable)
    pub fn yarn_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/yarn-releases/index.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);
        self
    }

    /// Setup mock to return the latest version of yarn (chainable)
    pub fn yarn_latest(mut self, version: &str) -> Self {
        let mock = mock("GET", "/yarn-latest")
            .with_status(200)
            .with_body(version)
            .create();
        self.root.mocks.push(mock);
        self
    }

    /// Setup mock to return the available npm versions (chainable)
    pub fn npm_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/registry/npm")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);

        self
    }

    /// Setup mock to return a 404 for any GET request
    /// Note: Mocks are matched in reverse order, so any created _after_ this will work
    ///       While those created before will not
    pub fn mock_not_found(mut self) -> Self {
        let mock = mock("GET", Matcher::Any).with_status(404).create();
        self.root.mocks.push(mock);
        self
    }

    fn distro_mock<T: DistroFixture>(mut self, fx: &T) -> Self {
        // ISSUE(#145): this should actually use a real http server instead of these mocks

        let server_path = fx.server_path();
        let fixture_path = fx.fixture_path();

        let metadata = fx.metadata();

        if let Some(uncompressed_size) = metadata.uncompressed_size {
            // This can be abstracted when https://github.com/rust-lang/rust/issues/52963 lands.
            let uncompressed_size_bytes: [u8; 4] = [
                ((uncompressed_size & 0xff000000) >> 24) as u8,
                ((uncompressed_size & 0x00ff0000) >> 16) as u8,
                ((uncompressed_size & 0x0000ff00) >> 8) as u8,
                (uncompressed_size & 0x000000ff) as u8,
            ];

            let range_mock = mock("GET", &server_path[..])
                .match_header("Range", Matcher::Any)
                .with_body(&uncompressed_size_bytes)
                .create();
            self.root.mocks.push(range_mock);
        }

        let file_mock = mock("GET", &server_path[..])
            .match_header("Range", Matcher::Missing)
            .with_header("Accept-Ranges", "bytes")
            .with_body_from_file(&fixture_path)
            .create();
        self.root.mocks.push(file_mock);

        self
    }

    pub fn distro_mocks<T: DistroFixture>(self, fixtures: &[DistroMetadata]) -> Self {
        let mut this = self;
        for fixture in fixtures {
            this = this.distro_mock::<T>(&fixture.clone().into());
        }
        this
    }

    /// Set a package config file for the sandbox (chainable)
    pub fn package_config(mut self, name: &str, contents: &str) -> Self {
        let package_cfg_file = package_config_file(name);
        self.files
            .push(FileBuilder::new(package_cfg_file, contents));
        self
    }

    /// Set a bin config file for the sandbox (chainable)
    pub fn binary_config(mut self, name: &str, contents: &str) -> Self {
        let bin_cfg_file = binary_config_file(name);
        self.files.push(FileBuilder::new(bin_cfg_file, contents));
        self
    }

    /// Set a shim file for the sandbox (chainable)
    pub fn shim(mut self, name: &str) -> Self {
        let shim_file = shim_file(name);
        self.files
            .push(FileBuilder::new(shim_file, "contents don't matter"));
        self
    }

    /// Set an unpackaged package for the sandbox (chainable)
    pub fn package_image(mut self, name: &str, version: &str) -> Self {
        let package_img_dir = package_image_dir(name, version);
        let package_json = package_img_dir.join("package.json");
        self.files.push(FileBuilder::new(
            package_json,
            &format!(r#"{{"name":"{}","version":"{}"}}"#, name, version),
        ));
        self
    }

    /// Set cached package tarballs for the sandbox (chainable)
    pub fn package_inventory(mut self, name: &str, version: &str) -> Self {
        let pkg_inventory_dir = package_inventory_dir();
        let package_tarball = pkg_inventory_dir.join(format!("{}-{}.tgz", name, version));
        self.files
            .push(FileBuilder::new(package_tarball, "tarball contents"));
        let package_shasum = pkg_inventory_dir.join(format!("{}-{}.shasum", name, version));
        self.files
            .push(FileBuilder::new(package_shasum, "shasum contents"));
        self
    }

    /// Create the project
    pub fn build(mut self) -> Sandbox {
        // First, clean the directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        // make sure these directories exist
        ok_or_panic! { fs::create_dir_all(node_cache_dir()) };
        ok_or_panic! { fs::create_dir_all(node_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(package_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(yarn_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(volta_tmp_dir()) };

        // Make sure the shims to npm and yarn exist
        ok_or_panic! { create_file_symlink(shim_exe(), self.root.npm_exe()) };
        ok_or_panic! { create_file_symlink(shim_exe(), self.root.yarn_exe()) };

        // write node and yarn caches
        for cache in self.caches.iter() {
            cache.build();
        }

        // write files
        for file_builder in self.files {
            file_builder.build();
        }

        // join dirs for the path (volta bin path is already first)
        self.root.path = env::join_paths(self.path_dirs.iter()).unwrap();

        let SandboxBuilder { root, .. } = self;
        root
    }

    fn rm_root(&self) {
        self.root.root().rm_rf()
    }
}

// files and dirs in the sandbox

fn home_dir() -> PathBuf {
    paths::home()
}
fn volta_home() -> PathBuf {
    home_dir().join(".volta")
}
fn volta_tmp_dir() -> PathBuf {
    volta_home().join("tmp")
}
fn volta_bin_dir() -> PathBuf {
    volta_home().join("bin")
}
fn volta_log_dir() -> PathBuf {
    volta_home().join("log")
}
fn volta_postscript() -> PathBuf {
    volta_tmp_dir().join("volta_tmp_1234.sh")
}
fn volta_tools_dir() -> PathBuf {
    volta_home().join("tools")
}
fn inventory_dir() -> PathBuf {
    volta_tools_dir().join("inventory")
}
fn user_dir() -> PathBuf {
    volta_tools_dir().join("user")
}
fn image_dir() -> PathBuf {
    volta_tools_dir().join("image")
}
fn node_inventory_dir() -> PathBuf {
    inventory_dir().join("node")
}
fn yarn_inventory_dir() -> PathBuf {
    inventory_dir().join("yarn")
}
fn package_inventory_dir() -> PathBuf {
    inventory_dir().join("packages")
}
fn cache_dir() -> PathBuf {
    volta_home().join("cache")
}
fn node_cache_dir() -> PathBuf {
    cache_dir().join("node")
}
#[allow(dead_code)]
fn node_index_file() -> PathBuf {
    node_cache_dir().join("index.json")
}
#[allow(dead_code)]
fn node_index_expiry_file() -> PathBuf {
    node_cache_dir().join("index.json.expires")
}
fn package_json_file(mut root: PathBuf) -> PathBuf {
    root.push("package.json");
    root
}
fn package_config_file(name: &str) -> PathBuf {
    user_dir().join("packages").join(format!("{}.json", name))
}
fn binary_config_file(name: &str) -> PathBuf {
    user_dir().join("bins").join(format!("{}.json", name))
}
fn shim_file(name: &str) -> PathBuf {
    volta_bin_dir().join(format!("{}{}", name, env::consts::EXE_SUFFIX))
}
fn package_image_dir(name: &str, version: &str) -> PathBuf {
    image_dir().join("packages").join(name).join(version)
}
fn user_platform_file() -> PathBuf {
    user_dir().join("platform.json")
}

fn sandbox_dir(dir_path: &str) -> PathBuf {
    home_dir().join(dir_path)
}

pub struct Sandbox {
    root: PathBuf,
    mocks: Vec<mockito::Mock>,
    env_vars: Vec<EnvVar>,
    env_vars_remove: Vec<String>,
    path: OsString,
}

impl Sandbox {
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
            // sandbox the Volta environment
            .env("VOLTA_HOME", volta_home())
            .env("PATH", &self.path)
            .env("VOLTA_POSTSCRIPT", volta_postscript())
            .env_remove("VOLTA_SHELL")
            .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows

        // overrides for env vars
        for env_var in &self.env_vars {
            p.env(&env_var.name, &env_var.value);
        }

        for env_var_name in &self.env_vars_remove {
            p.env_remove(env_var_name);
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

    /// Create a `ProcessBuilder` to run the volta npm shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.npm("install ember-cli"), execs());
    pub fn npm(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.npm_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn npm_exe(&self) -> PathBuf {
        self.root().join(format!("npm{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run the volta yarn shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.yarn("add ember-cli"), execs());
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.yarn_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn yarn_exe(&self) -> PathBuf {
        self.root().join(format!("yarn{}", env::consts::EXE_SUFFIX))
    }

    pub fn read_package_json(&self) -> String {
        let package_file = package_json_file(self.root());
        read_file_to_string(package_file)
    }

    pub fn read_postscript(&self) -> String {
        let postscript_file = volta_postscript();
        read_file_to_string(postscript_file)
    }

    pub fn read_log_dir(&self) -> Option<fs::ReadDir> {
        fs::read_dir(volta_log_dir()).ok()
    }

    pub fn remove_volta_home(&self) -> () {
        volta_home().rm_rf();
    }

    // check that files in the sandbox exist

    pub fn node_inventory_archive_exists(&self, version: &str) -> bool {
        node_inventory_dir()
            .join(node_distro_file_name(version))
            .exists()
    }

    pub fn yarn_inventory_archive_exists(&self, version: &str) -> bool {
        yarn_inventory_dir()
            .join(yarn_distro_file_name(version))
            .exists()
    }

    pub fn package_config_exists(name: &str) -> bool {
        package_config_file(name).exists()
    }
    pub fn bin_config_exists(name: &str) -> bool {
        binary_config_file(name).exists()
    }
    pub fn shim_exists(name: &str) -> bool {
        shim_file(name).exists()
    }
    pub fn dir_exists(dir_path: &str) -> bool {
        sandbox_dir(dir_path).exists()
    }
    pub fn package_image_exists(name: &str, version: &str) -> bool {
        let package_img_dir = package_image_dir(name, version);
        package_img_dir.join("package.json").exists()
    }
    pub fn pkg_inventory_tarball_exists(name: &str, version: &str) -> bool {
        let pkg_inventory_dir = package_inventory_dir();
        pkg_inventory_dir
            .join(format!("{}-{}.tgz", name, version))
            .exists()
    }
    pub fn pkg_inventory_shasum_exists(name: &str, version: &str) -> bool {
        let pkg_inventory_dir = package_inventory_dir();
        pkg_inventory_dir
            .join(format!("{}-{}.shasum", name, version))
            .exists()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        self.root().rm_rf();
    }
}

// Generates a sandboxed environment
pub fn sandbox() -> SandboxBuilder {
    SandboxBuilder::new(paths::root().join("sandbox"))
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

pub fn shim_exe() -> PathBuf {
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

fn read_file_to_string(file_path: PathBuf) -> String {
    let mut contents = String::new();
    let mut file = ok_or_panic! { File::open(file_path) };
    ok_or_panic! { file.read_to_string(&mut contents) };
    contents
}
