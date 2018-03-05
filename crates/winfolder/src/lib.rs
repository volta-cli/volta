//! This crate provides access to Windows APIs for querying the location of standard
//! [standard Windows folders](https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911.aspx)
//! on the current system.
//!
//! # Example
//!
//! ```
//! extern crate winfolder;
//!
//! use winfolder::Folder;
//!
//! # fn main() {
//! # let _ =
//! Folder::ProgramFilesX86.path()
//! # ;
//! # }
//! ```

#![cfg(windows)]

extern crate winapi;
extern crate shell32;
extern crate ole32;

#[macro_use]
extern crate guid;

pub mod id;

use id::*;

use std::ptr::null_mut;
use std::mem;
use std::ffi::OsString;
use std::slice;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use winapi::winnt::PWSTR;
use winapi::minwindef::MAX_PATH;
use shell32::SHGetKnownFolderPath;
use ole32::CoTaskMemFree;

/// Construct an `OsString` from a pointer to a wide-character string.
/// This is marked as unsafe because it can only be safely used on a
/// string that is known to come from a trusted API.
unsafe fn os_string_from_trusted_api(mut p: PWSTR) -> OsString {
    let mut s: OsString = OsString::with_capacity(MAX_PATH + 1);
    while *p != 0 {
        s.push(&OsString::from_wide(slice::from_raw_parts(p, 1)));
        p = p.offset(1);
    }
    s
}

/// Returns the path for a Windows
/// [known folder](https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911.aspx)
/// based on that folder's GUID. Some standard known folder GUIDs can be found in
/// the `winfolder::id` module.
///
/// If the GUID does not represent a standard folder, this function
/// produces `None`.
///
/// This function provides the functionality of the standard Windows
/// [SHGetKnownFolderPath](https://msdn.microsoft.com/en-us/library/windows/desktop/bb762188.aspx)
/// API.
pub fn known_path(guid: &guid::GUID) -> Option<PathBuf> {
    let string: OsString;
    unsafe {
        let mut path: PWSTR = null_mut();
        if SHGetKnownFolderPath(guid, 0, null_mut(), mem::transmute(&mut path)) != 0 {
            return None;
        }
        string = os_string_from_trusted_api(path);
        CoTaskMemFree(mem::transmute(path));
    }
    Some(Path::new(&string).to_path_buf())
}

/// Represents a standard Windows [known folder](https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911.aspx).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Folder {
    /// The [`FOLDERID_LocalAppData`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_localappdata)
    /// known folder.
    LocalAppData,

    /// The [`FOLDERID_ProgramData`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programdata)
    /// known folder.
    ProgramData,

    /// The [`FOLDERID_ProgramFiles`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfiles)
    /// known folder.
    ProgramFiles,

    /// The [`FOLDERID_ProgramFilesX64`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx64)
    /// known folder.
    ProgramFilesX64,

    /// The [`FOLDERID_ProgramFilesX86`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx86)
    /// known folder.
    ProgramFilesX86
}

impl Folder {

    /// Returns the Windows GUID associated with this known folder.
    pub fn id(self) -> guid::GUID {
        match self {
            Folder::LocalAppData    => LOCAL_APP_DATA,
            Folder::ProgramData     => PROGRAM_DATA,
            Folder::ProgramFiles    => PROGRAM_FILES,
            Folder::ProgramFilesX64 => PROGRAM_FILES_X64,
            Folder::ProgramFilesX86 => PROGRAM_FILES_X86
        }
    }

    /// Returns the path for this known folder on this system.
    ///
    /// This function provides the functionality of the standard Windows
    /// [SHGetKnownFolderPath](https://msdn.microsoft.com/en-us/library/windows/desktop/bb762188.aspx)
    /// API.
    pub fn path(self) -> PathBuf {
        known_path(&self.id()).expect("Folder::path")
    }

}

#[cfg(test)]
mod tests {
    use super::{known_path, Folder};
    use super::id;
    use std::path::Path;

    #[test]
    fn it_works() {
        assert_eq!(known_path(&id::PROGRAM_FILES_X86), Some(Path::new(r"C:\Program Files (x86)").to_path_buf()));
        assert_eq!(Folder::ProgramFilesX86.path(), Path::new(r"C:\Program Files (x86)").to_path_buf());
    }
}
