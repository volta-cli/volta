//! The main implementation crate for the core of Volta.

// The `doc_cfg` feature has to be enabled for platform-specific API doc generation.
// https://doc.rust-lang.org/nightly/unstable-book/language-features/doc-cfg.html
#![cfg_attr(feature = "cross-platform-docs", feature(doc_cfg))]

mod command;
#[cfg(not(feature = "volta-updates"))]
pub mod env;
pub mod error;
mod event;
pub mod fs;
mod hook;
pub mod inventory;
pub mod layout;
pub mod log;
pub mod manifest;
pub mod monitor;
pub mod platform;
pub mod project;
pub mod run;
pub mod session;
#[cfg(not(feature = "volta-updates"))]
pub mod shell;
pub mod shim;
pub mod signal;
pub mod style;
pub mod tool;
pub mod toolchain;
pub mod version;
