use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use reqwest::header::HttpDate;

use support;
use support::paths::{self, PathExt};
use support::process::ProcessBuilder;

#[cfg(feature = "mock-network")]
use mockito::{self, mock, Matcher};

// package.json
#[derive(PartialEq, Clone)]
struct PackageBuilder {
    path: PathBuf,
    contents: String,
}

impl PackageBuilder {
    pub fn new(path: PathBuf, contents: &str) -> PackageBuilder {
        PackageBuilder {
            path,
            contents: contents.to_string(),
        }
    }

    pub fn build(&self) {
        self.dirname().mkdir_p();

        let mut file = File::create(&self.path).unwrap_or_else(|e| {
            panic!(
                "could not create package.json file {}: {}",
                self.path.display(),
                e
            )
        });

        t!(file.write_all(self.contents.as_bytes()));
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

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
        t!(cache_file.write_all(self.contents.as_bytes()));

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
        t!(expiry_file.write_all(expiry_date.to_string().as_bytes()));
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

pub struct Sandbox {
    root: PathBuf,
    mocks: Vec<mockito::Mock>,
}

#[must_use]
pub struct SandboxBuilder {
    root: Sandbox,
    package: Option<PackageBuilder>,
    caches: Vec<CacheBuilder>,
    node_index_mock: Option<String>,
    yarn_index_mock: Option<String>,
    yarn_latest_mock: Option<String>,
    node_archive_mock: Option<String>,
    yarn_archive_mock: Option<String>,
    // more TODO
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
            },
            package: None,
            caches: vec![],
            node_index_mock: None,
            yarn_index_mock: None,
            yarn_latest_mock: None,
            node_archive_mock: None,
            yarn_archive_mock: None,
        }
    }

    /// Set the Node cache for the sandbox (chainable)
    #[allow(dead_code)]
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
        self.package = Some(PackageBuilder::new(package_file, contents));
        self
    }

    /// Create the project
    pub fn build(mut self) -> Sandbox {
        // First, clean the directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        // write package.json
        if let Some(package_builder) = self.package {
            package_builder.build();
        } else {
            default_package(self.root()).build();
        }

        // TODO: make this cleaner, somehow?
        t!(fs::create_dir_all(node_cache_dir()));
        t!(fs::create_dir_all(yarn_cache_dir()));

        // write node and yarn caches
        for cache in self.caches.iter() {
            cache.build();
        }

        // setup network mocks
        if let Some(_mock) = self.node_index_mock {
            panic!("unimplemented!!"); // TODO
        } else {
            self.root.mocks.push(default_node_index_mock());
        }

        if let Some(_mock) = self.yarn_index_mock {
            panic!("unimplemented!!"); // TODO
        } else {
            self.root.mocks.push(default_yarn_index_mock());
        }

        if let Some(_mock) = self.yarn_latest_mock {
            panic!("unimplemented!!"); // TODO
        } else {
            self.root.mocks.push(default_yarn_latest_mock());
        }

        if let Some(_mock) = self.node_archive_mock {
            panic!("unimplemented!!"); // TODO
        } else {
            self.root.mocks.append(&mut default_node_archive_mocks());
        }

        if let Some(_mock) = self.yarn_archive_mock {
            panic!("unimplemented!!"); // TODO
        } else {
            self.root.mocks.append(&mut default_yarn_archive_mocks());
        }

        let SandboxBuilder { root, .. } = self;
        root
    }

    fn rm_root(&self) {
        self.root.root().rm_rf()
    }
}

// files and dirs in the sandbox
// TODO: some of these are different on windows?

fn home_dir() -> PathBuf {
    paths::home()
}
fn notion_home() -> PathBuf {
    home_dir().join(".notion")
}
fn cache_dir() -> PathBuf {
    notion_home().join("cache")
}
fn node_cache_dir() -> PathBuf {
    cache_dir().join("node")
}
fn yarn_cache_dir() -> PathBuf {
    cache_dir().join("yarn")
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

fn fixture_dir() -> PathBuf {
    let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_manifest_dir.push("tests");
    cargo_manifest_dir.push("testsuite");
    cargo_manifest_dir.push("fixtures");
    cargo_manifest_dir
}

fn fixture_file(file: &str) -> PathBuf {
    let mut path = fixture_dir();
    path.push(file);
    path
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
        let mut p = support::process::process(program);
        p.cwd(self.root())
            // sandbox the Notion environment
            // TODO: allow overrides for these
            .env("HOME", home_dir())
            .env("NOTION_HOME", notion_home())
            .env("PATH", "")
            .env_remove("NOTION_DEV")
            .env_remove("NOTION_NODE_VERSION")
            .env_remove("NOTION_POSTSCRIPT")
            .env_remove("NOTION_SHELL");
        // TODO: need this on windows?
        // .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows
        p
    }

    /// Create a `ProcessBuilder` to run notion.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.notion("use node 9.5"), execs());
    pub fn notion(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&notion_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn read_package_json(&self) -> String {
        let package_file = package_json_file(self.root());
        let mut contents = String::new();
        let mut file = t!(File::open(package_file));
        t!(file.read_to_string(&mut contents));
        contents
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

fn notion_exe() -> PathBuf {
    cargo_dir().join(format!("notion{}", env::consts::EXE_SUFFIX))
}

// default file contents

fn default_package(mut root: PathBuf) -> PackageBuilder {
    root.push("package.json");
    let contents = r#"{
  "name": "default-test-package",
  "version": "1.7.3",
  "description": "Default description",
  "author": "Default Person <default.person@zombo.com>",
  "main": "index.js"
}
        "#;
    PackageBuilder::new(root, contents)
}

// default network mocks

fn default_node_index_mock() -> mockito::Mock {
    mock("GET", "/node-dist/index.json") // TODO make that a constant
        // .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[
{"version":"v10.8.0","date":"2018-08-15","files":["aix-ppc64","headers","linux-arm64","linux-armv6l","linux-armv7l","linux-ppc64le","linux-x64","osx-x64-pkg","osx-x64-tar","src","sunos-x64","win-x64-7z","win-x64-exe","win-x64-msi","win-x64-zip","win-x86-7z","win-x86-exe","win-x86-msi","win-x86-zip"],"npm":"6.2.0","v8":"6.8.275.24","uv":"1.22.0","zlib":"1.2.11","openssl":"1.1.0i","modules":"64","lts":false},
{"version":"v9.11.2","date":"2018-06-12","files":["aix-ppc64","headers","linux-arm64","linux-armv6l","linux-armv7l","linux-ppc64le","linux-x64","linux-x86","osx-x64-pkg","osx-x64-tar","src","sunos-x64","sunos-x86","win-x64-7z","win-x64-exe","win-x64-msi","win-x64-zip","win-x86-7z","win-x86-exe","win-x86-msi","win-x86-zip"],"npm":"5.6.0","v8":"6.2.414.46","uv":"1.19.2","zlib":"1.2.11","openssl":"1.0.2o","modules":"59","lts":false}
        ]"#
        )
        .create()
}

fn default_yarn_index_mock() -> mockito::Mock {
    mock("GET", "/yarn-releases/index.json") // TODO make that a constant
        // .with_status(200)
        // .with_header("content-type", "application/json")
        .with_body(r#"[ "1.0.0", "1.0.1", "1.2.0", "1.4.0", "1.9.2", "1.9.4" ]"#)
        .create()
}

fn default_yarn_latest_mock() -> mockito::Mock {
    mock("GET", "/yarn-latest") // TODO make that a constant
        // .with_status(200)
        // .with_header("content-type", "application/json")
        .with_body("1.2.0")
        .create()
}

fn default_node_archive_mocks() -> Vec<mockito::Mock> {
    let mut mocks = Vec::new();

    // the mock archive file
    // TODO: this will be different on windows (tgz vs zip)
    let archive_file_mock = fixture_file("test-archive.tar.gz");
    let mut f = t!(File::open(archive_file_mock));
    let mut buffer = Vec::new();
    // read the whole file, as bytes
    t!(f.read_to_end(&mut buffer));
    // convert to string
    let file_string = String::from_utf8_lossy(&buffer);

    // the HEAD request, to get the file size
    mocks.push(
        mock(
            "HEAD",
            Matcher::Regex(r"^/v\d+.\d+.\d+/node-v\d+.\d+.\d+".to_string()),
        ).with_header("Accept-Ranges", "bytes")
            .with_body(&file_string)
            .create(),
    );

    // for the "Range: bytes" request, to get the ISIZE value (last 4 bytes)
    // this will be interpreted as a packed integer value
    // (doesn't really matter - used for progress bar)
    let isize_info = "1234";
    mocks.push(
        mock(
            "GET",
            Matcher::Regex(r"^/v\d+.\d+.\d+/node-v\d+.\d+.\d+".to_string()),
        ).match_header("Range", Matcher::Any)
            .with_body(&isize_info)
            .create(),
    );

    // the actual file
    mocks.push(
        mock(
            "GET",
            Matcher::Regex(r"^/v\d+.\d+.\d+/node-v\d+.\d+.\d+".to_string()),
        ).match_header("Range", Matcher::Missing)
            .with_body(&file_string)
            .create(),
    );

    mocks
}

fn default_yarn_archive_mocks() -> Vec<mockito::Mock> {
    let mut mocks = Vec::new();

    // the mock archive file
    // TODO: this will be different on windows (tgz vs zip)
    let archive_file_mock = fixture_file("test-archive.tar.gz");
    let mut f = t!(File::open(archive_file_mock));
    let mut buffer = Vec::new();
    // read the whole file, as bytes
    t!(f.read_to_end(&mut buffer));
    // convert to string
    let file_string = String::from_utf8_lossy(&buffer);

    // the HEAD request, to get the file size
    mocks.push(
        mock("HEAD", Matcher::Regex(r"^/yarn-v\d+.\d+.\d+".to_string()))
            .with_header("Accept-Ranges", "bytes")
            .with_body(&file_string)
            .create(),
    );

    // for the "Range: bytes" request, to get the ISIZE value (last 4 bytes)
    // this will be interpreted as a packed integer value
    // (doesn't really matter - used for progress bar)
    let isize_info = "1234";
    mocks.push(
        mock("GET", Matcher::Regex(r"^/yarn-v\d+.\d+.\d+".to_string()))
            .match_header("Range", Matcher::Any)
            .with_body(&isize_info)
            .create(),
    );

    // the actual file
    mocks.push(
        mock("GET", Matcher::Regex(r"^/yarn-v\d+.\d+.\d+".to_string()))
            .match_header("Range", Matcher::Missing)
            .with_body(&file_string)
            .create(),
    );

    mocks
}

fn split_and_add_args(p: &mut ProcessBuilder, s: &str) {
    for arg in s.split_whitespace() {
        if arg.contains('"') || arg.contains('\'') {
            panic!("shell-style argument parsing is not supported")
        }
        p.arg(arg);
    }
}
