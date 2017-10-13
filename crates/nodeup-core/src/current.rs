use config::{self, Config};
use version::Version;

pub enum Which {
    Local,
    Global,
    System
}

pub fn get(which: Option<Which>) -> ::Result<String> {
    match which {
        Some(Which::Local) => {
            let Config { node: Version::Public(version) } = config::read()?;
            Ok(version)
        }
        Some(Which::Global) => {
            unimplemented!()
        }
        Some(Which::System) => {
            unimplemented!()
        }
        None => {
            // FIXME: print out all three
            unimplemented!()
        }
    }
}