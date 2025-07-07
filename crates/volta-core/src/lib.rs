//! The main implementation crate for the core of Volta.

use std::{env, ffi::OsString};

mod command;
pub mod error;
pub mod event;
pub mod fs;
mod hook;
pub mod inventory;
pub mod layout;
pub mod log;
pub mod monitor;
pub mod platform;
pub mod project;
pub mod run;
pub mod session;
pub mod shim;
pub mod signal;
pub mod style;
pub mod sync;
pub mod tool;
pub mod toolchain;
pub mod version;

const VOLTA_FEATURE_PNPM: &str = "VOLTA_FEATURE_PNPM";
const VOLTA_FEATURE_YARN: &str = "VOLTA_FEATURE_YARN";

pub fn is_yarn_enabled() -> bool {
    env::var_os(VOLTA_FEATURE_YARN) != Some(OsString::from("0"))
}
