extern crate notion_core;

use notion_core::tool::{Node, Tool};

/// The entry point for the `node` shim.
pub fn main() {
    Node::launch()
}
