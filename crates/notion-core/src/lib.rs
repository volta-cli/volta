//! The main implementation crate for the core of Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

extern crate cmdline_words_parser;
extern crate console;
extern crate detect_indent;
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
mod distro;
pub mod env;
mod event;
pub mod manifest;
pub mod monitor;
pub mod path;
mod plugin;
pub mod project;
pub mod serial;
pub mod session;
pub mod shell;
pub mod shim;
pub mod style;
pub mod tool;

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate notion_fail;
#[macro_use]
extern crate notion_fail_derive;

#[macro_use]
extern crate cfg_if;

use std::fs;
use std::io;
use std::path::Path;

use notion_fail::{ExitCode, Fallible, NotionFail, ResultExt};

#[derive(Debug, Fail, NotionFail)]
#[fail(display = "Could not create directory {}: {}", dir, error)]
#[notion_fail(code = "FileSystemError")]
pub(crate) struct CreateDirError {
    pub(crate) dir: String,
    pub(crate) error: String,
}

impl CreateDirError {
    pub(crate) fn for_dir(dir: String) -> impl FnOnce(&io::Error) -> CreateDirError {
        move |error| CreateDirError {
            dir,
            error: error.to_string(),
        }
    }
}

/// If the input path is a directory, it creates that directory. If the input path is a file, it creates the parent directory of that file.
pub fn ensure_dir_exists<P: AsRef<Path>>(path: &P) -> Fallible<()> {
    let p = path.as_ref();
    if p.is_dir() {
        fs::create_dir_all(p).with_context(CreateDirError::for_dir(p.to_string_lossy().to_string()))
    } else if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)
            .with_context(CreateDirError::for_dir(dir.to_string_lossy().to_string()))
    } else {
        Ok(())
    }
}
