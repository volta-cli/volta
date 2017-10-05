use std::ptr::null_mut;
use std::mem;
use std::ffi::OsString;
use std::slice;
use std::os::windows::ffi::OsStringExt;

use winapi::guiddef::GUID;
use winapi::winnt::PWSTR;
use winapi::minwindef::MAX_PATH;
use shell32::SHGetKnownFolderPath;
use ole32::CoTaskMemFree;

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_localappdata
// {F1B32785-6FBA-4FCF-9D55-7B8E7F157091}
const GUID_FOLDERID_LOCALAPPDATA: GUID = GUID {
    Data1: 0xF1B32785,
    Data2: 0x6FBA,
    Data3: 0x4FCF,
    Data4: [0x9D, 0x55, 0x7B, 0x8E, 0x7F, 0x15, 0x70, 0x91]
};

unsafe fn os_string_from_trusted_api(mut p: PWSTR) -> OsString {
    let mut s: OsString = OsString::with_capacity(MAX_PATH + 1);
    while *p != 0 {
        s.push(&OsString::from_wide(slice::from_raw_parts(p, 1)));
        p = p.offset(1);
    }
    s
}

pub fn get_local_app_data_path() -> OsString {
    let string: OsString;
    unsafe {
        let mut path: PWSTR = null_mut();
        SHGetKnownFolderPath(&GUID_FOLDERID_LOCALAPPDATA, 0, null_mut(), mem::transmute(&mut path));
        string = os_string_from_trusted_api(path);
        CoTaskMemFree(mem::transmute(path));
    }
    string
}
