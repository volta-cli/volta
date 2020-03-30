use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        mod support;

        // test files
        mod corrupted_download;
        mod intercept_global_installs;
        mod merged_platform;
        mod migrations;
        mod run_shim_directly;
        mod verbose_errors;
        mod volta_bypass;
        mod volta_install;
        mod volta_pin;
        mod volta_uninstall;
    }
}
