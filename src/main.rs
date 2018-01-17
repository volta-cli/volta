#[macro_use]
extern crate serde_derive;
extern crate notion_core;
extern crate docopt;
extern crate console;

mod command;

fn main() {
    command::run();
}
