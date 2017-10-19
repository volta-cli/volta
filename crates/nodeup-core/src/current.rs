//use config::{self, Config};
use version::Version;

pub enum Which {
    Local,
    Global,
    System
}

pub fn get(which: Option<Which>) -> ::Result<Option<String>> {
    match which {
        Some(Which::Local) => {
            //Ok(config::read_local()?.map(|Config { node: Version::Public(version) }| version))
            unimplemented!()
        }
        Some(Which::Global) => {
            //let Config { node: Version::Public(version) } = config::read_global()?;
            //Ok(Some(version))
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
