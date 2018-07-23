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

pub mod catalog;
pub mod config;
pub mod env;
mod event;
mod installer;
pub mod manifest;
pub mod monitor;
mod package_info;
pub mod path;
mod plugin;
pub mod shell;
pub mod project;
pub mod serial;
pub mod session;
pub mod style;
pub mod tool;

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate notion_fail;

#[macro_use]
extern crate cfg_if;
