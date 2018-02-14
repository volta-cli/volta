extern crate notion_core;

use notion_core::tool::{Tool, Binary};

/// The entry point for shims to third-party binary executables.
pub fn main() {
    Binary::launch()
}
