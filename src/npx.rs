extern crate notion_core;

use notion_core::tool::{Tool, Npx};

/// The entry point for the `npx` shim.
pub fn main() {
    Npx::launch()
}
