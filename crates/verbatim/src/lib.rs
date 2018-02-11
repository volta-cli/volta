use std::path::{Path, PathBuf, Prefix, Component};

pub trait PathExt {
    fn to_verbatim(&self) -> PathBuf;
}

impl PathExt for Path {
    fn to_verbatim(&self) -> PathBuf {
        let mut components = self.components();
        match components.next() {
            Some(Component::Prefix(prefix)) => {
                let new_prefix = match prefix.kind() {
                    // FIXME: fill out all these cases
                    Prefix::Verbatim(_str) => { unimplemented!() }
                    Prefix::VerbatimUNC(_hostname, _sharename) => { unimplemented!() }
                    Prefix::VerbatimDisk(_letter) => { unimplemented!() }
                    Prefix::DeviceNS(_devicename) => { unimplemented!() }
                    Prefix::UNC(_hostname, _sharename) => { unimplemented!() }
                    Prefix::Disk(letter) => {
                        let new_prefix_string = format!(r"\\?\{}:\", String::from_utf8_lossy(&[letter]));
                        let new_prefix = Path::new(&new_prefix_string).to_path_buf();
                        new_prefix
                    }
                };
                new_prefix.join(components)
            }
            Some(other) => {
                Path::new(r"\\?\").join(Path::new(&other)).join(components)
            }
            None => {
                // FIXME: handle this
                panic!("corner case");
            }
        }
    }
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
