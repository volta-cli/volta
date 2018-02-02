use std::path::Path;
use std::fs::{File, remove_dir_all};
use std::io::{self, Write};
use std::str::FromStr;

use toml::value::{Value, Table};

use version::Version;
use path::{self, user_catalog_file};
use untoml::{ParseToml, Extract, load};
use provision;
use failure;

use super::CatalogError;

fn toml_error(key: String) -> CatalogError {
    CatalogError {
        msg: format!("invalid catalog file at key '{}'", key)
    }
}

pub struct Catalog {
    pub node: Option<Version>
}

pub enum Installed {
    Already,
    Now
}

impl Catalog {

    pub fn current() -> Result<Catalog, failure::Error> {
        let path = user_catalog_file()?;
        load(&path)
    }

    fn save(&self, path: &Path) -> Result<(), failure::Error> {
        let mut file = File::create(path)?;
        if let Some(Version::Public(ref version)) = self.node {
            file.write_all(b"[node]\n")?;
            file.write_fmt(format_args!("version = \"{}\"\n", version))?;
        }
        Ok(())
    }

    pub fn set_version(&mut self, version: Version) -> Result<(), failure::Error> {
        {
            let &Version::Public(ref version) = &version;
            self.install(version)?;
        }
        self.node = Some(version);
        self.save(&user_catalog_file()?)?;
        Ok(())
    }

    pub fn install(&mut self, version: &str) -> Result<Installed, failure::Error> {
        // FIXME: this should be based on the data structure instead
        if path::node_version_dir(version)?.is_dir() {
            Ok(Installed::Already)
        } else {
            let dest = path::node_versions_dir()?;
            provision::by_version(&dest, version)?;
            // FIXME: update the data structure and self.save()
            Ok(Installed::Now)
        }
    }

    pub fn uninstall(&mut self, version: &str) -> Result<(), failure::Error> {
        let home = path::node_version_dir(version)?;

        // FIXME: this should be based on the data structure instead
        if !home.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a directory", home.to_string_lossy())))?;
        }

        remove_dir_all(home)?;

        // FIXME: update the data structure and self.save()

        Ok(())
    }

}

fn parse_node_version(root: &mut Table) -> Result<Option<Version>, failure::Error> {
    if !root.contains_key("node") {
        return Ok(None);
    }
    let mut node = root.extract("node", toml_error)?.table("node", toml_error)?;
    if !node.contains_key("version") {
        return Ok(None);
    }
    let version = node.extract("version", toml_error)?.string("node.version", toml_error)?;
    Ok(Some(Version::Public(version)))
}

impl FromStr for Catalog {
    type Err = failure::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let toml = src.parse::<Value>()?;
        let mut root = toml.table("<root>", toml_error)?;
        let node = parse_node_version(&mut root)?;
        Ok(Catalog { node })
    }
}
