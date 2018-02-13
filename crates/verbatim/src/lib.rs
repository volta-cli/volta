//! This crate provides a trait for converting a `Path` into a "verbatim path,"
//! which is Rust's terminology for a Windows
//! [_extended-length path_](https://msdn.microsoft.com/en-us/library/windows/desktop/aa365247(v=vs.85).aspx#maxpath).
//! Because of Windows's legacy limit of 260 characters for traditional filesystem paths,
//! modern versions of Windows also support an alternative path format, constructed with
//! the prefix `\\?\`, which supports much longer paths. Rust refers to these paths as
//! _verbatim paths_.
//!
//! **It is important to understand that Windows does not normalize `/` characters for
//! verbatim paths.** When converting a path to a verbatim path, your logic must take
//! responsibility for any normalization you might need to perform before calling any
//! Windows APIs or serializing a path to interact with an external system.

use std::path::{Path, PathBuf, Prefix, Component};

/// An extension to the `std::path::Path` type to provide a method for constructing an
/// equivalent verbatim path.
pub trait PathExt {
    /// Converts the path to a verbatim path.
    fn to_verbatim(&self) -> PathBuf;
}

impl PathExt for Path {
    /// Converts a `Path` to a verbatim path.
    fn to_verbatim(&self) -> PathBuf {
        let mut components = self.components();
        match components.next() {
            Some(Component::Prefix(prefix)) => {
                let new_prefix = match prefix.kind() {
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
                unimplemented!()
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
