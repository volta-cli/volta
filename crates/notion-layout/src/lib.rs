extern crate notion_layout_macro;
extern crate cfg_if;

pub mod v0;
pub mod v1;

use std::path::{Path, PathBuf};
use cfg_if::cfg_if;

pub(crate) fn executable(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}
