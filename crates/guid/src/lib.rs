#![cfg(windows)]

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


/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
*/
