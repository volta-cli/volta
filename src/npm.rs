extern crate notion_core;

use notion_core::tool::{Npm, CmdTool};

/// The entry point for the `npm` shim.
pub fn main() {
    Npm::launch()
}
