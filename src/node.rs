extern crate notion_core;

use notion_core::tool::{CmdTool, Node};

/// The entry point for the `node` shim.
pub fn main() {
    Node::launch()
}
