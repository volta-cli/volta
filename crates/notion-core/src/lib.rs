//! The main implementation crate for the core of Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

#[cfg(feature = "mock-network")]
extern crate mockito;

pub mod config;
mod distro;
pub mod env;
mod event;
pub(crate) mod fs;
pub mod inventory;
pub mod manifest;
pub mod monitor;
pub mod path;
pub mod platform;
mod plugin;
pub mod project;
pub mod session;
pub mod shell;
pub mod shim;
pub mod style;
pub mod tool;
pub mod toolchain;
pub mod version;

use failure;

#[macro_use]
extern crate notion_fail;
#[macro_use]
extern crate notion_fail_derive;

#[macro_use]
extern crate cfg_if;
