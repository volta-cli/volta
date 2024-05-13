use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use cfg_if::cfg_if;
use headers::{Expires, Header};
use mockito::{self, mock, Matcher};
use node_semver::Version;
use test_support::{self, ok_or_panic, paths, paths::PathExt, process::ProcessBuilder};
use volta_core::fs::{set_executable, symlink_file};
use volta_core::tool::{Node, Pnpm, Yarn};

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
        let expiry_date = Expires::from(if self.expired {
            SystemTime::now() - one_day
        } else {
            SystemTime::now() + one_day
        });

        let mut header_values = Vec::with_capacity(1);
        expiry_date.encode(&mut header_values);
        // Since we just `.encode()`d into `header_values, it is guaranteed to
        // have a `.first()`.
        let encoded_expiry_date = header_values.first().unwrap();

        let mut expiry_file = File::create(&self.expiry_path).unwrap_or_else(|e| {
            panic!(
                "could not create cache expiry file {}: {}",
                self.expiry_path.display(),
                e
            )
        });
        ok_or_panic! { expiry_file.write_all(encoded_expiry_date.as_bytes()) };
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

// used to construct sandboxed files like package.json, platform.json, etc.
#[derive(PartialEq, Eq, Clone)]
pub struct FileBuilder {
    path: PathBuf,
    contents: String,
    executable: bool,
}

impl FileBuilder {
    pub fn new(path: PathBuf, contents: &str) -> FileBuilder {
        FileBuilder {
            path,
            contents: contents.to_string(),
            executable: false,
        }
    }

    pub fn make_executable(mut self) -> Self {
        self.executable = true;
        self
    }

    pub fn build(&self) {
        self.dirname().mkdir_p();

        let mut file = File::create(&self.path)
            .unwrap_or_else(|e| panic!("could not create file {}: {}", self.path.display(), e));

        ok_or_panic! { file.write_all(self.contents.as_bytes()) };
        if self.executable {
            ok_or_panic! { set_executable(&self.path) };
        }
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

struct ShimBuilder {
    name: String,
}

impl ShimBuilder {
    fn new(name: String) -> ShimBuilder {
        ShimBuilder { name }
    }

    fn build(&self) {
        ok_or_panic! { symlink_file(shim_exe(), shim_file(&self.name)) };
    }
}

// used to setup executable binaries in installed packages
pub struct PackageBinInfo {
    pub name: String,
    pub contents: String,
}

#[must_use]
pub struct SandboxBuilder {
    root: Sandbox,
    files: Vec<FileBuilder>,
    caches: Vec<CacheBuilder>,
    path_dirs: Vec<PathBuf>,
    shims: Vec<ShimBuilder>,
    has_exec_path: bool,
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

pub struct NpmFixture {
    pub metadata: DistroMetadata,
}

pub struct PnpmFixture {
    pub metadata: DistroMetadata,
}

pub struct Yarn1Fixture {
    pub metadata: DistroMetadata,
}

pub struct YarnBerryFixture {
    pub metadata: DistroMetadata,
}

impl From<DistroMetadata> for NodeFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl From<DistroMetadata> for NpmFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl From<DistroMetadata> for PnpmFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl From<DistroMetadata> for Yarn1Fixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl From<DistroMetadata> for YarnBerryFixture {
    fn from(metadata: DistroMetadata) -> Self {
        Self { metadata }
    }
}

impl DistroFixture for NodeFixture {
    fn server_path(&self) -> String {
        let version = Version::parse(self.metadata.version).unwrap();
        let filename = Node::archive_filename(&version);
        format!("/v{version}/{filename}")
    }

    fn fixture_path(&self) -> String {
        let version = Version::parse(self.metadata.version).unwrap();
        let filename = Node::archive_filename(&version);
        format!("tests/fixtures/{filename}")
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl DistroFixture for NpmFixture {
    fn server_path(&self) -> String {
        format!("/npm/-/npm-{}.tgz", self.metadata.version)
    }

    fn fixture_path(&self) -> String {
        format!("tests/fixtures/npm-{}.tgz", self.metadata.version)
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl DistroFixture for PnpmFixture {
    fn server_path(&self) -> String {
        format!("/pnpm/-/pnpm-{}.tgz", self.metadata.version)
    }

    fn fixture_path(&self) -> String {
        format!("tests/fixtures/pnpm-{}.tgz", self.metadata.version)
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl DistroFixture for Yarn1Fixture {
    fn server_path(&self) -> String {
        format!("/yarn/-/yarn-{}.tgz", self.metadata.version)
    }

    fn fixture_path(&self) -> String {
        format!("tests/fixtures/yarn-{}.tgz", self.metadata.version)
    }

    fn metadata(&self) -> &DistroMetadata {
        &self.metadata
    }
}

impl DistroFixture for YarnBerryFixture {
    fn server_path(&self) -> String {
        format!(
            "/@yarnpkg/cli-dist/-/cli-dist-{}.tgz",
            self.metadata.version
        )
    }

    fn fixture_path(&self) -> String {
        format!("tests/fixtures/cli-dist-{}.tgz", self.metadata.version)
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
            shims: vec![
                ShimBuilder::new("npm".to_string()),
                ShimBuilder::new("pnpm".to_string()),
                ShimBuilder::new("yarn".to_string()),
            ],
            has_exec_path: false,
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
            .push(FileBuilder::new(default_platform_file(), contents));
        self
    }

    /// Set the hooks.json for the sandbox
    pub fn default_hooks(mut self, contents: &str) -> Self {
        self.files
            .push(FileBuilder::new(default_hooks_file(), contents));
        self
    }

    /// Set a layout version file for the sandbox (chainable)
    pub fn layout_file(mut self, version: &str) -> Self {
        self.files.push(FileBuilder::new(layout_file(version), ""));
        self
    }
    /// Set an environment variable for the sandbox (chainable)
    pub fn env(mut self, name: &str, value: &str) -> Self {
        self.root.env_vars.push(EnvVar::new(name, value));
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

    /// Setup mock to return the available Yarn@1 versions (chainable)
    pub fn yarn_1_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/yarn")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);
        self
    }

    /// Setup mock to return the available Yarn@2+ versions (chainable)
    pub fn yarn_berry_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/@yarnpkg/cli-dist")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);
        self
    }

    /// Setup mock to return the available npm versions (chainable)
    pub fn npm_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/npm")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        self.root.mocks.push(mock);

        self
    }

    /// Setup mock to return the available pnpm versions (chainable)
    pub fn pnpm_available_versions(mut self, body: &str) -> Self {
        let mock = mock("GET", "/pnpm")
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
                ((uncompressed_size & 0xff00_0000) >> 24) as u8,
                ((uncompressed_size & 0x00ff_0000) >> 16) as u8,
                ((uncompressed_size & 0x0000_ff00) >> 8) as u8,
                (uncompressed_size & 0x0000_00ff) as u8,
            ];

            let range_mock = mock("GET", &server_path[..])
                .match_header("Range", Matcher::Any)
                .with_body(uncompressed_size_bytes)
                .create();
            self.root.mocks.push(range_mock);
        }

        let file_mock = mock("GET", &server_path[..])
            .match_header("Range", Matcher::Missing)
            .with_header("Accept-Ranges", "bytes")
            .with_body_from_file(fixture_path)
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

    /// Add an arbitrary file to the sandbox (chainable)
    pub fn file(mut self, path: &str, contents: &str) -> Self {
        let file_name = sandbox_path(path);
        self.files.push(FileBuilder::new(file_name, contents));
        self
    }

    /// Add an arbitrary file to the test project within the sandbox (chainable)
    pub fn project_file(mut self, path: &str, contents: &str) -> Self {
        let file_name = self.root().join(path);
        self.files.push(FileBuilder::new(file_name, contents));
        self
    }

    /// Add an arbitrary file to the test project within the sandbox,
    /// give it executable permissions,
    /// and add its directory to the PATH
    /// (chainable)
    pub fn executable_file(mut self, path: &str, contents: &str) -> Self {
        let file_name = self.root().join("exec").join(path);
        self.files
            .push(FileBuilder::new(file_name, contents).make_executable());
        self.add_exec_dir_to_path()
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
        self.shims.push(ShimBuilder::new(name.to_string()));
        self
    }

    /// Set an unpackaged package for the sandbox (chainable)
    pub fn package_image(
        mut self,
        name: &str,
        version: &str,
        bins: Option<Vec<PackageBinInfo>>,
    ) -> Self {
        let package_img_dir = package_image_dir(name);
        let package_json = package_img_dir.join("package.json");
        self.files.push(FileBuilder::new(
            package_json,
            &format!(r#"{{"name":"{}","version":"{}"}}"#, name, version),
        ));
        if let Some(bin_infos) = bins {
            for bin_info in bin_infos.iter() {
                cfg_if! {
                    if #[cfg(target_os = "windows")] {
                        let bin_path = package_img_dir.join(format!("{}.cmd", &bin_info.name));
                    } else {
                        let bin_path = package_img_dir.join("bin").join(&bin_info.name);
                    }
                }
                self.files
                    .push(FileBuilder::new(bin_path, &bin_info.contents).make_executable());
            }
        }
        self
    }

    /// Write executable project binaries into node_modules/.bin/ (chainable)
    pub fn project_bins(mut self, bins: Vec<PackageBinInfo>) -> Self {
        let project_bin_dir = self.root().join("node_modules").join(".bin");
        for bin_info in bins.iter() {
            cfg_if! {
                if #[cfg(target_os = "windows")] {
                    // in Windows, binaries have an extra file with an executable extension
                    let win_bin_path = project_bin_dir.join(format!("{}.cmd", &bin_info.name));
                    self.files.push(FileBuilder::new(win_bin_path, &bin_info.contents).make_executable());
                }
            }
            // Volta on both Windows and Unix checks for the existence of the binary with no extension
            let bin_path = project_bin_dir.join(&bin_info.name);
            self.files
                .push(FileBuilder::new(bin_path, &bin_info.contents).make_executable());
        }
        self
    }

    /// Write '.pnp.cjs' file in local project to mark as Plug-n-Play (chainable)
    pub fn project_pnp(mut self) -> Self {
        let pnp_path = self.root().join(".pnp.cjs");
        self.files.push(FileBuilder::new(pnp_path, "blegh"));
        self
    }

    /// Write an executable node binary with the input contents (chainable)
    pub fn setup_node_binary(
        mut self,
        node_version: &str,
        npm_version: &str,
        contents: &str,
    ) -> Self {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let node_file = "node.cmd";
            } else {
                let node_file = "node";
            }
        }
        let node_bin_file = node_image_dir(node_version).join("bin").join(node_file);
        self.files
            .push(FileBuilder::new(node_bin_file, contents).make_executable());
        self.node_npm_version_file(node_version, npm_version)
    }

    /// Write an executable npm binary with the input contents (chainable)
    pub fn setup_npm_binary(mut self, version: &str, contents: &str) -> Self {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let npm_file = "npm.cmd";
            } else {
                let npm_file = "npm";
            }
        }
        let npm_bin_file = npm_image_dir(version).join("bin").join(npm_file);
        self.files
            .push(FileBuilder::new(npm_bin_file, contents).make_executable());
        self
    }

    /// Write an executable pnpm binary with the input contents (chainable)
    pub fn setup_pnpm_binary(mut self, version: &str, contents: &str) -> Self {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let pnpm_file = "pnpm.cmd";
            } else {
                let pnpm_file = "pnpm";
            }
        }
        let pnpm_bin_file = pnpm_image_dir(version).join("bin").join(pnpm_file);
        self.files
            .push(FileBuilder::new(pnpm_bin_file, contents).make_executable());
        self
    }

    /// Write an executable yarn binary with the input contents (chainable)
    pub fn setup_yarn_binary(mut self, version: &str, contents: &str) -> Self {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let yarn_file = "yarn.cmd";
            } else {
                let yarn_file = "yarn";
            }
        }
        let yarn_bin_file = yarn_image_dir(version).join("bin").join(yarn_file);
        self.files
            .push(FileBuilder::new(yarn_bin_file, contents).make_executable());
        self
    }

    /// Write the "default npm" file for a node version (chainable)
    pub fn node_npm_version_file(mut self, node_version: &str, npm_version: &str) -> Self {
        let npm_file = node_npm_version_file(node_version);
        self.files.push(FileBuilder::new(npm_file, npm_version));
        self
    }

    /// Add directory to the PATH (chainable)
    pub fn add_dir_to_path(mut self, dir: PathBuf) -> Self {
        self.path_dirs.push(dir);
        self
    }

    /// Add executable directory to the PATH (chainable)
    pub fn add_exec_dir_to_path(mut self) -> Self {
        if !self.has_exec_path {
            let exec_path = self.root().join("exec");
            self.path_dirs.push(exec_path);
            self.has_exec_path = true;
        }
        self
    }

    /// Create the project
    pub fn build(mut self) -> Sandbox {
        // First, clean the directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        // make sure these directories exist
        ok_or_panic! { fs::create_dir_all(volta_bin_dir()) };
        ok_or_panic! { fs::create_dir_all(node_cache_dir()) };
        ok_or_panic! { fs::create_dir_all(node_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(package_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(pnpm_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(yarn_inventory_dir()) };
        ok_or_panic! { fs::create_dir_all(volta_tmp_dir()) };

        // write node and yarn caches
        for cache in self.caches.iter() {
            cache.build();
        }

        // write files
        for file_builder in self.files {
            file_builder.build();
        }

        // write shims
        for shim_builder in self.shims {
            shim_builder.build();
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
fn pnpm_inventory_dir() -> PathBuf {
    inventory_dir().join("pnpm")
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
fn package_image_dir(name: &str) -> PathBuf {
    image_dir().join("packages").join(name)
}
fn node_image_dir(version: &str) -> PathBuf {
    image_dir().join("node").join(version)
}
fn npm_image_dir(version: &str) -> PathBuf {
    image_dir().join("npm").join(version)
}
fn pnpm_image_dir(version: &str) -> PathBuf {
    image_dir().join("pnpm").join(version)
}
fn yarn_image_dir(version: &str) -> PathBuf {
    image_dir().join("yarn").join(version)
}
fn default_platform_file() -> PathBuf {
    user_dir().join("platform.json")
}
fn default_hooks_file() -> PathBuf {
    volta_home().join("hooks.json")
}
fn layout_file(version: &str) -> PathBuf {
    volta_home().join(format!("layout.{}", version))
}
fn node_npm_version_file(node_version: &str) -> PathBuf {
    node_inventory_dir().join(format!("node-v{}-npm", node_version))
}

fn sandbox_path(path: &str) -> PathBuf {
    home_dir().join(path)
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
            .env("VOLTA_INSTALL_DIR", cargo_dir())
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
        let mut p = self.process(volta_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Create a `ProcessBuilder` to run the volta npm shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.npm("install ember-cli"), execs());
    pub fn npm(&self, cmd: &str) -> ProcessBuilder {
        self.exec_shim("npm", cmd)
    }

    /// Create a `ProcessBuilder` to run the volta pnpm shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.pnpm("add ember-cli"), execs());
    pub fn pnpm(&self, cmd: &str) -> ProcessBuilder {
        self.exec_shim("pnpm", cmd)
    }

    /// Create a `ProcessBuilder` to run the volta yarn shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.yarn("add ember-cli"), execs());
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        self.exec_shim("yarn", cmd)
    }

    /// Create a `ProcessBuilder` to run an arbitrary shim.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.exec_shim("cowsay", "foo bar"), execs());
    pub fn exec_shim(&self, bin: &str, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(shim_file(bin));
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn read_package_json(&self) -> String {
        let package_file = package_json_file(self.root());
        read_file_to_string(package_file)
    }

    pub fn read_log_dir(&self) -> Option<fs::ReadDir> {
        fs::read_dir(volta_log_dir()).ok()
    }

    pub fn remove_volta_home(&self) {
        volta_home().rm_rf();
    }

    // check that files in the sandbox exist

    pub fn node_inventory_archive_exists(&self, version: &Version) -> bool {
        node_inventory_dir()
            .join(Node::archive_filename(version))
            .exists()
    }

    pub fn pnpm_inventory_archive_exists(&self, version: &str) -> bool {
        pnpm_inventory_dir()
            .join(Pnpm::archive_filename(version))
            .exists()
    }

    pub fn yarn_inventory_archive_exists(&self, version: &str) -> bool {
        yarn_inventory_dir()
            .join(Yarn::archive_filename(version))
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
    pub fn path_exists(path: &str) -> bool {
        sandbox_path(path).exists()
    }
    pub fn package_image_exists(name: &str) -> bool {
        let package_img_dir = package_image_dir(name);
        package_img_dir.join("package.json").exists()
    }
    pub fn read_default_platform() -> String {
        read_file_to_string(default_platform_file())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        paths::root().rm_rf();
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
