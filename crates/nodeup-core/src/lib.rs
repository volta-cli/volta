extern crate indicatif;
extern crate term_size;
extern crate toml;
extern crate node_archive;

#[cfg(windows)]
extern crate winfolder;

pub mod config;
pub mod provision;
pub mod install;
pub mod uninstall;
pub mod path;
pub mod launch;
pub mod version;

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
