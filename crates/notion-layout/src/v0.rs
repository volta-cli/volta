use crate::executable;
use std::path::PathBuf;

use notion_layout_macro::layout;

layout! {
    /// The V0 layout schema for Notion.
    pub struct NotionHome {
        "cache": cache_dir {
            "node": node_cache_dir {
                "index.json": node_index_file;
                "index.json.expires": node_index_expiry_file;
            }
            "yarn": yarn_cache_dir { }
        }
        "versions": versions_dir {
            "node": node_versions_dir { }
            "yarn": yarn_versions_dir { }
        }
        "bin": shim_dir { }
        "notion.exe": notion_file;
        "launchbin.exe": launchbin_file;
        "launchscript.exe": launchscript_file;
        "config.toml": user_config_file;
        "catalog.toml": user_catalog_file;
    }
}

impl NotionHome {
    pub fn node_version_dir(&self, version: &str) -> PathBuf {
        self.node_versions_dir().join(version)
    }

    pub fn node_version_bin_dir(&self, version: &str) -> PathBuf {
        if cfg!(windows) {
            self.node_version_dir(version)
        } else {
            self.node_version_dir(version).join("bin")
        }
    }

    pub fn node_version_3p_bin_dir(&self, version: &str) -> PathBuf {
        if cfg!(windows) {
            // ISSUE (#90): Figure out where binaries are globally installed on Windows
            unimplemented!("global 3rd party executables not yet implemented for Windows")
        } else {
            self.node_version_dir(version).join("lib").join("node_modules").join(".bin")
        }
    }

    pub fn yarn_version_dir(&self, version: &str) -> PathBuf {
        self.yarn_versions_dir().join(version)
    }

    pub fn yarn_version_bin_dir(&self, version: &str) -> PathBuf {
        self.yarn_version_dir(version).join("bin")
    }

    pub fn shim_file(&self, toolname: &str) -> PathBuf {
        self.shim_dir().join(&executable(toolname))
    }
}
