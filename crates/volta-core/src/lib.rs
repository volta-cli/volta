//! The main implementation crate for the core of Volta.

// The `doc_cfg` feature has to be enabled for platform-specific API doc generation.
// https://doc.rust-lang.org/nightly/unstable-book/language-features/doc-cfg.html
#![cfg_attr(feature = "cross-platform-docs", feature(doc_cfg))]

mod command;
mod distro;
pub mod env;
pub mod error;
mod event;
pub(crate) mod fetch;
pub(crate) mod fs;
mod hook;
pub mod inventory;
pub mod log;
pub mod manifest;
pub mod monitor;
pub mod path;
pub mod platform;
pub mod project;
pub(crate) mod resolve;
pub mod run;
pub mod session;
pub mod shell;
pub mod shim;
pub mod style;
pub mod tool;
pub mod toolchain;
pub mod version;
