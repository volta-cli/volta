#![cfg(windows)]

//! A macro for defining Windows GUIDs with syntax similar to the one used
//! conventionally, e.g. in tools like Visual Studio's `guidgen.exe`.
//! 
//! # Example
//! 
//! ```
//! #[macro_use]
//! extern crate guid;
//! extern crate winapi;
//! 
//! use winapi::guiddef::GUID;
//! 
//! /// The GUID for the `%windir%\system32` folder (`{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}`).
//! pub const SYSTEM32_FOLDER: GUID = guid!(0x1AC14E77, 0x02E7, 0x4E5D, 0xB744, 0x2EB1AE5198B7);
//! ```

extern crate winapi;

pub use winapi::guiddef::GUID;

#[macro_export]
macro_rules! guid {
    ($chunk1:expr, $chunk2:expr, $chunk3:expr, $chunk4:expr, $chunk5:expr) => {
        $crate::GUID {
            Data1: $chunk1,
            Data2: $chunk2,
            Data3: $chunk3,
            Data4: [
                ((($chunk4 as u16) & 0xFF00            ) >>        8 ) as u8,
                ((($chunk4 as u16) & 0x00FF            )             ) as u8,

                ((($chunk5 as u64) & 0x0000FF0000000000) >> (32 +  8)) as u8,
                ((($chunk5 as u64) & 0x000000FF00000000) >> (32 -  0)) as u8,
                ((($chunk5 as u64) & 0x00000000FF000000) >> (32 -  8)) as u8,
                ((($chunk5 as u64) & 0x0000000000FF0000) >> (32 - 16)) as u8,
                ((($chunk5 as u64) & 0x000000000000FF00) >> (32 - 24)) as u8,
                ((($chunk5 as u64) & 0x00000000000000FF) >> (32 - 32)) as u8
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use winapi::guiddef::GUID;

    #[test]
    fn system32_folder() {
        assert_eq!(GUID {
            Data1: 0x1AC14E77,
            Data2: 0x02E7,
            Data3: 0x4E5D,
            Data4: [0xB744, 0x2EB1, 0xAE51, 0x98B7]
        },
        guid!(0x1AC14E77, 0x02E7, 0x4E5D, 0xB744, 0x2EB1AE5198B7),
        "FOLDERID_system GUID");
    }
}
