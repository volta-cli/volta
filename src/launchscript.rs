extern crate notion_core;

use notion_core::tool::{Script, CmdTool};

/// The entry point for shims to third-party scripts.
pub fn main() {
    Script::launch()
}
