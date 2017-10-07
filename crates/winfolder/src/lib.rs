#![cfg(windows)]

extern crate winapi;
extern crate shell32;
extern crate ole32;

#[macro_use]
extern crate guid;

pub mod id;

use std::ptr::null_mut;
use std::mem;
use std::ffi::OsString;
use std::slice;
use std::os::windows::ffi::OsStringExt;

use winapi::winnt::PWSTR;
use winapi::minwindef::MAX_PATH;
use shell32::SHGetKnownFolderPath;
use ole32::CoTaskMemFree;

unsafe fn os_string_from_trusted_api(mut p: PWSTR) -> OsString {
    let mut s: OsString = OsString::with_capacity(MAX_PATH + 1);
    while *p != 0 {
        s.push(&OsString::from_wide(slice::from_raw_parts(p, 1)));
        p = p.offset(1);
    }
    s
}

// FIXME: safely handle errors
pub fn known_path(guid: &guid::GUID) -> OsString {
    let string: OsString;
    unsafe {
        let mut path: PWSTR = null_mut();
        SHGetKnownFolderPath(guid, 0, null_mut(), mem::transmute(&mut path));
        string = os_string_from_trusted_api(path);
        CoTaskMemFree(mem::transmute(path));
    }
    string
}


/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
