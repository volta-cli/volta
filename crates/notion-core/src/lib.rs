//! The main implementation crate for the core of Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

extern crate cmdline_words_parser;
extern crate console;
extern crate indicatif;
extern crate lazycell;
extern crate node_archive;
extern crate readext;
extern crate reqwest;
extern crate semver;
extern crate serde_json;
extern crate tempfile;
extern crate term_size;
extern crate toml;

extern crate serde;
#[macro_use]
extern crate serde_derive;

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
mod plugin;
mod installer;

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate notion_fail;

#[macro_use]
extern crate cfg_if;
