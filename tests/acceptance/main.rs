#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate hamcrest2;
#[cfg(feature = "mock-network")]
extern crate mockito;

#[macro_use]
extern crate test_support;

mod support;

// test files

mod intercept_global_installs;
mod notion_current;
mod notion_deactivate;
mod notion_pin;
