mod support;

// test files

#[cfg(feature = "intercept-globals")]
mod intercept_global_installs;
mod notion_current;
mod notion_deactivate;
mod notion_pin;
#[macro_use] // to use the assert_that! macro
extern crate hamcrest2;
