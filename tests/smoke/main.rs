#[macro_use]
extern crate cfg_if;
extern crate envoy;
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

cfg_if! {
    if #[cfg(all(unix, feature = "smoke-tests"))] {
        mod notion_fetch;
        mod notion_install;
        mod autodownload;
    }
}
