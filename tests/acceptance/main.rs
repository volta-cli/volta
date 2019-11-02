mod support;

// test files

#[cfg(not(feature = "volta-updates"))]
mod autocreate_home_dir;
mod corrupted_download;
mod intercept_global_installs;
mod merged_platform;
#[cfg(feature = "volta-updates")]
mod migrations;
#[cfg(feature = "volta-updates")]
mod run_shim_directly;
mod verbose_errors;
mod volta_bypass;
mod volta_current;
#[cfg(not(feature = "volta-updates"))]
mod volta_deactivate;
mod volta_pin;
mod volta_uninstall;
