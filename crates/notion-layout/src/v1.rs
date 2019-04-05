use crate::executable;
use std::path::{Path, PathBuf};

use notion_layout_macro::layout;

pub struct NotionLayout {
    pub install: NotionInstall,
    pub user: NotionHome,
}

impl NotionLayout {
    pub fn new(install: PathBuf, home: PathBuf) -> Self {
        NotionLayout {
            install: NotionInstall::new(install),
            user: NotionHome::new(home),
        }
    }

    pub fn create(&self) -> ::std::io::Result<()> {
        self.install.create()?;
        self.user.create()?;
        Ok(())
    }

    pub fn env_paths(&self) -> Vec<PathBuf> {
        if cfg!(windows) {
            vec![
                self.user.shim_dir().to_path_buf(),
                self.install.bin_dir().to_path_buf()
            ]
        } else {
            vec![self.user.shim_dir().to_path_buf()]
        }
    }
}

layout! {
    /// The V1 layout for the core Notion installation directory.
    pub struct NotionInstall {
        "notion[.exe]": notion_file;
        "shim[.exe]": shim_executable;
    }

    /// The V1 layout schema for the Notion user home directory.
    pub struct NotionHome {
        ".schema": schema_file;
        "cache": cache_dir {
            "node": node_cache_dir {
                "index.json": node_index_file;
                "index.json.expires": node_index_expiry_file;
            }
        }
        "bin": shim_dir { }
        "log": log_dir { }
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
                "bins": user_tool_bin_dir { }
                "packages": user_package_dir { }
                "platform.json": user_platform_file;
            }
        }
        "tmp": tmp_dir { }
        "hooks.json": user_hooks_file;
    }
}

fn package_distro_file_name(name: &str, version: &str) -> String {
    format!("{}.tgz", package_basename(name, version))
}

fn package_shasum_file_name(name: &str, version: &str) -> String {
    format!("{}.shasum", package_basename(name, version))
}

fn package_basename(name: &str, version: &str) -> String {
    format!("{}-{}", name, version)
}

impl NotionInstall {
    pub fn bin_dir(&self) -> &Path {
        // FIXME: should be <root>\bin on Windows
        self.root()
    }
}

impl NotionHome {
    pub fn node_npm_version_file(&self, version: &str) -> PathBuf {
        let filename = format!("node-v{}-npm", version);
        self.node_inventory_dir().join(&filename)
    }

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

    pub fn yarn_image_dir(&self, version: &str) -> PathBuf {
        self.yarn_image_root_dir().join(version)
    }

    pub fn yarn_image_bin_dir(&self, version: &str) -> PathBuf {
        self.yarn_image_dir(version).join("bin")
    }

    pub fn shim_file(&self, toolname: &str) -> PathBuf {
        self.shim_dir().join(&executable(toolname))
    }

    pub fn shim_git_bash_file(&self, toolname: &Ustr) -> PathBuf {
        self.shim_dir().join(toolname)
    }

    pub fn package_image_dir(&self, name: &str, version: &str) -> PathBuf {
        self.package_image_root_dir().join(name).join(version)
    }

    pub fn package_distro_file(&self, name: &str, version: &str) -> PathBuf {
        self.package_inventory_dir()
            .join(package_distro_file_name(name, version))
    }

    pub fn package_distro_shasum(&self, name: &str, version: &str) -> PathBuf {
        self.package_inventory_dir()
            .join(package_shasum_file_name(name, version))
    }

    pub fn user_package_config_file(&self, package_name: &str) -> PathBuf {
        self.user_package_dir()
            .join(format!("{}.json", package_name))
    }

    pub fn user_tool_bin_config(&self, bin_name: &str) -> PathBuf {
        self.user_tool_bin_dir().join(format!("{}.json", bin_name))
    }
}
