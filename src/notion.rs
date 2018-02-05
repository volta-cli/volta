#[macro_use]
extern crate serde_derive;
extern crate notion_core;
extern crate docopt;
extern crate console;
extern crate failure;
extern crate semver;

mod command;

fn main() {
    command::run();
}
