use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        mod support;

        // test files
        mod corrupted_download;
        mod direct_install;
        mod direct_uninstall;
        mod execute_binary;
        mod hooks;
        mod merged_platform;
        mod migrations;
        mod run_shim_directly;
        mod verbose_errors;
        mod volta_bypass;
        mod volta_install;
        mod volta_pin;
        mod volta_run;
        mod volta_uninstall;
    }
}
