#[macro_use]
extern crate cfg_if;
extern crate failure;
#[macro_use]
extern crate hamcrest2;
#[cfg(feature = "mock-network")]
extern crate mockito;
extern crate notion_core;
extern crate notion_fail;
extern crate rand;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate test_support;

mod support;

// test files

mod notion_current;
mod notion_deactivate;
mod notion_use;
