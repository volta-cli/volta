#[macro_use]
extern crate serde_derive;
extern crate notion_core;
extern crate docopt;
extern crate console;
extern crate failure;
extern crate semver;

mod command;

/// The entry point for the `notion` CLI.
pub fn main() {
    command::run();
}
