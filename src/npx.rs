extern crate notion_core;

use notion_core::tool::{Npx, CmdTool};

/// The entry point for the `npx` shim.
pub fn main() {
    Npx::launch()
}
