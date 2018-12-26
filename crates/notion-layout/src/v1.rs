use crate::executable;
use std::path::PathBuf;

use notion_layout_macro::layout;

layout! {
    /// The V1 layout schema for Notion.
    pub struct NotionHome {
        ".schema": schema_file;
        "cache": cache_dir {
            "node": node_cache_dir {
                "index.json": node_index_file;
                "index.json.expires": node_index_expiry_file;
            }
        }
        "bin": shim_dir { }
        "notion.exe": notion_file;
        "launchbin.exe": launchbin_file;
        "launchscript.exe": launchscript_file;
        "config.toml": user_config_file;
        "shell": shell_dir { }
        "tools": tools_dir {
            "inventory": inventory_dir {
                "node": node_inventory_dir { }
                "packages": package_inventory_dir { }
                "yarn": yarn_inventory_dir { }
            }
            "image": image_dir {
                "node": node_image_root_dir { }
                "packages": package_image_root_dir { }
                "yarn": yarn_image_root_dir { }
            }
            "user": user_toolchain_dir {
                "bins": user_tool_binaries_dir { }
                "packages": user_tool_packages_dir { }
                "platform.json": user_platform_file;
            }
        }
    }
}

impl NotionHome {
    pub fn node_image_dir(&self, node: &str, npm: &str) -> PathBuf {
        self.node_image_root_dir().join(node).join(npm)
    }

    pub fn node_image_bin_dir(&self, node: &str, npm: &str) -> PathBuf {
        if cfg!(windows) {
            self.node_image_dir(node, npm)
        } else {
            self.node_image_dir(node, npm).join("bin")
        }
    }

    pub fn node_image_3p_bin_dir(&self, node: &str, npm: &str) -> PathBuf {
        if cfg!(windows) {
            // ISSUE (#90): Figure out where binaries are globally installed on Windows
            unimplemented!("global 3rd party executables not yet implemented for Windows")
        } else {
            self.node_image_dir(node, npm).join("lib").join("node_modules").join(".bin")
        }
    }

    pub fn yarn_image_dir(&self, version: &str) -> PathBuf {
        self.yarn_image_root_dir().join(version)
    }

    pub fn yarn_image_bin_dir(&self, version: &str) -> PathBuf {
        self.yarn_image_dir(version).join("bin")
    }

    pub fn shim_file(&self, toolname: &str) -> PathBuf {
        self.shim_dir().join(&executable(toolname))
    }
}
