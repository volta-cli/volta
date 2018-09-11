#[macro_use]
extern crate cfg_if;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(feature = "mock-network")]
extern crate mockito;
extern crate notion_core;
#[macro_use]
extern crate notion_fail;
#[macro_use]
extern crate notion_fail_derive;
extern crate rand;
extern crate reqwest;
// #[macro_use]
extern crate serde_json;

#[macro_use]
mod support;

// test files

mod notion_current;
mod notion_deactivate;
mod notion_use;
