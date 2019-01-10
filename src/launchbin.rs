extern crate notion_core;

use notion_core::tool::{Binary, CmdTool};

/// The entry point for shims to third-party binary executables.
pub fn main() {
    Binary::launch()
}
