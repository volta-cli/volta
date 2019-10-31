use std::path::PathBuf;

use volta_layout::v0::VoltaHome;

pub struct V0 {
    pub home: VoltaHome,
}

impl V0 {
    pub fn new(home: PathBuf) -> Self {
        V0 {
            home: VoltaHome::new(home),
        }
    }
}
