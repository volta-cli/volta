//! The main implementation crate for the core of Notion.

extern crate indicatif;
extern crate term_size;
extern crate toml;
extern crate node_archive;
extern crate serde_json;
extern crate console;
extern crate lazycell;
extern crate readext;
extern crate semver;
extern crate cmdline_words_parser;
extern crate reqwest;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(windows)]
extern crate winfolder;

pub mod path;
pub mod env;
pub mod config;
pub mod tool;
pub mod project;
pub mod manifest;
pub mod catalog;
pub mod session;
pub mod style;
pub mod serial;
pub mod error;
mod plugin;
mod installer;

#[macro_use]
extern crate failure_derive;
extern crate failure;

#[macro_use]
extern crate cfg_if;
