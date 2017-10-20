// needed for `error_chain!` macro
#![recursion_limit = "1024"]

extern crate indicatif;
extern crate term_size;
extern crate toml;
extern crate node_archive;
extern crate serde_json;

#[macro_use]
extern crate error_chain;

#[cfg(windows)]
extern crate winfolder;

pub mod path;
pub mod provision;
pub mod install;
pub mod uninstall;
pub mod env;
pub mod launch;
pub mod version;
pub mod current;
pub mod project;
pub mod manifest;
pub mod lockfile;
pub mod global;
mod untoml;

use std::process::exit;

mod errors {
    use node_archive;
    use toml;

    error_chain! {
        links {
            Archive(node_archive::Error, node_archive::ErrorKind);
        }

        foreign_links {
            Toml(toml::de::Error);
            Io(::std::io::Error);
            Json(::serde_json::error::Error);
        }

        errors {
            ConfigError(key: String) {
                description("error in configuration")
                display("error in configuration key '{}'", key)
            }
            UnknownSystemFolder(name: String) {
                description("unknown system folder")
                display("unknown system folder: '{}'", name)
            }
            VersionParseError(src: String) {
                description("invalid version specifier")
                display("invalid version specifier: {}", src)
            }
            ManifestError(msg: String) {
                description("manifest error")
                display("invalid manifest: {}", msg)
            }
            LockfileError(msg: String) {
                description("lockfile error")
                display("invalid lockfile: {}", msg)
            }
        }
    }
}

pub use errors::*;

pub fn display_error(err: ::Error) {
    // FIXME: polish the error reporting
    eprintln!("error: {}", err);

    for err in err.iter().skip(1) {
        eprintln!("\tcaused by: {}", err);
    }

    if let Some(backtrace) = err.backtrace() {
        eprintln!("backtrace: {:?}", backtrace);
    }
}

pub fn die(err: ::Error) -> ! {
    display_error(err);
    exit(1);
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
