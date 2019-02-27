//! The main implementation crate for the core of Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

mod distro;
pub mod env;
pub mod error;
mod event;
pub(crate) mod fs;
mod hook;
pub mod inventory;
pub mod manifest;
pub mod monitor;
pub mod package;
pub mod path;
pub mod platform;
pub mod project;
pub mod session;
pub mod shell;
pub mod shim;
pub mod style;
pub mod tool;
pub mod toolchain;
pub mod version;
