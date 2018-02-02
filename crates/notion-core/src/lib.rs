extern crate indicatif;
extern crate term_size;
extern crate toml;
extern crate node_archive;
extern crate serde_json;
extern crate console;

#[cfg(windows)]
extern crate winfolder;

pub mod path;
pub mod provision;
pub mod env;
pub mod config;
pub mod launch;
pub mod version;
pub mod project;
pub mod manifest;
pub mod lockfile;
pub mod catalog;
pub mod style;
mod untoml;

#[macro_use]
extern crate failure_derive;
extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "Error in configuration key '{}'", key)]
pub struct ConfigError {
    key: String
}

#[derive(Fail, Debug)]
#[fail(display = "Notion has encountered an internal error ('{}')", msg)]
pub struct CatalogError {
    msg: String
}

#[derive(Fail, Debug)]
#[fail(display = "Unknown system folder: '{}'", name)]
pub struct UnknownSystemFolderError {
    name: String
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid version specifier: '{}'", src)]
pub struct VersionParseError {
    src: String
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid manifest: {}", msg)]
pub struct ManifestError {
    msg: String
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid lockfile: {}", msg)]
pub struct LockfileError {
    msg: String
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
