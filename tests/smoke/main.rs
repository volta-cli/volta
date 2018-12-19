/// Smoke tests for Notion, that will be run in CI.
///
/// To run these locally:
/// (CAUTION: this will destroy the Notion installation on the system where this is run)
///
/// ```
/// NOTION_DEV=1 cargo test --test smoke --features smoke-tests -- --test-threads 1
/// ```
///
/// Also note that each test uses a different version of node and yarn. This is to prevent
/// false positives if the tests are not cleaned up correctly. Any new tests should use
/// different versions of node and yarn.
#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(all(unix, feature = "smoke-tests"))] {
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
        mod notion_fetch;
        mod notion_install;
        mod autodownload;
    }
}
