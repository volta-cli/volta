// Smoke tests for Volta, that will be run in CI.
//
// To run these locally:
// (CAUTION: this will destroy the Volta installation on the system where this is run)
//
// ```
// VOLTA_LOGLEVEL=debug cargo test --test smoke --features smoke-tests -- --test-threads 1
// ```
//
// Also note that each test uses a different version of node and yarn. This is to prevent
// false positives if the tests are not cleaned up correctly. Any new tests should use
// different versions of node and yarn.

cfg_if::cfg_if! {
    if #[cfg(all(unix, feature = "smoke-tests"))] {
        mod autodownload;
        mod direct_install;
        mod direct_upgrade;
        mod npm_link;
        mod package_migration;
        pub mod support;
        mod volta_fetch;
        mod volta_install;
        mod volta_run;
    }
}
