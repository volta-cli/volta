use std::path::PathBuf;

use volta_layout::v0::VoltaHome;

/// Represents a V0 Volta layout (from before v0.7.0)
///
/// This needs some migration work to move up to V1, so we keep a reference to the V0 layout
/// struct to allow for easy comparison between versions
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
