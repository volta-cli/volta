use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json;

use notion_core::path;

use test_support::{self, paths, paths::PathExt, process::ProcessBuilder};

// catalog.toml
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

        ok_or_panic!{ file.write_all(self.contents.as_bytes()) };
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

        // make sure these directories exist and are empty
        ok_or_panic!(path::node_cache_dir()).ensure_empty();
        ok_or_panic!(path::shim_dir()).ensure_empty();
        ok_or_panic!(path::node_inventory_dir()).ensure_empty();
        ok_or_panic!(path::yarn_inventory_dir()).ensure_empty();
        ok_or_panic!(path::package_inventory_dir()).ensure_empty();
        ok_or_panic!(path::node_image_root_dir()).ensure_empty();
        ok_or_panic!(path::yarn_image_root_dir()).ensure_empty();
        ok_or_panic!(path::user_toolchain_dir()).ensure_empty();
        ok_or_panic!(path::tmp_dir()).ensure_empty();
        // and these files do not exist
        ok_or_panic!(path::notion_file()).rm();
        ok_or_panic!(path::launchbin_file()).rm();
        ok_or_panic!(path::launchscript_file()).rm();
        ok_or_panic!(path::user_config_file()).rm();
        ok_or_panic!(path::user_platform_file()).rm();

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
            // setup the Notion environment
            .env_remove("NOTION_NODE_VERSION")
            .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows

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

    /// Create a `ProcessBuilder` to run Node.
    pub fn node(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&node_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Create a `ProcessBuilder` to run Yarn.
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&yarn_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Verify that the input Node version has been fetched.
    pub fn node_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = path::node_distro_file_name(version);
        let inventory_dir = ok_or_panic!{ path::node_inventory_dir() };
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Node version has been unpacked.
    pub fn node_version_is_unpacked(&self, version: &str, npm_version: &str) -> bool {
        let unpack_dir = ok_or_panic!{ path::node_image_bin_dir(version, npm_version) };
        unpack_dir.exists()
    }

    /// Verify that the input Node version has been installed.
    pub fn assert_node_version_is_installed(&self, version: &str, npm_version: &str) -> () {
        let user_platform = ok_or_panic!{ path::user_platform_file() };
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["runtime"], version);
        assert_eq!(json_contents["node"]["npm"], npm_version);
    }

    /// Verify that the input Yarn version has been fetched.
    pub fn yarn_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = path::yarn_distro_file_name(version);
        let inventory_dir = ok_or_panic!{ path::yarn_inventory_dir() };
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Yarn version has been unpacked.
    pub fn yarn_version_is_unpacked(&self, version: &str) -> bool {
        let unpack_dir = ok_or_panic!{ path::yarn_image_dir(version) };
        unpack_dir.exists()
    }

    /// Verify that the input Yarn version has been installed.
    pub fn assert_yarn_version_is_installed(&self, version: &str) -> () {
        let user_platform = ok_or_panic!{ path::user_platform_file() };
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["yarn"], version);
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

fn notion_exe() -> PathBuf {
    cargo_dir().join(format!("notion{}", env::consts::EXE_SUFFIX))
}

fn node_exe() -> PathBuf {
    cargo_dir().join(format!("node{}", env::consts::EXE_SUFFIX))
}

fn yarn_exe() -> PathBuf {
    cargo_dir().join(format!("yarn{}", env::consts::EXE_SUFFIX))
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
    let mut file = ok_or_panic!{ File::open(file_path) };
    ok_or_panic!{ file.read_to_string(&mut contents) };
    contents
}
