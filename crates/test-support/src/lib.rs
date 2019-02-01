//! Utilities to use with acceptance tests in Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

use failure;

#[macro_use]
extern crate notion_fail;
#[macro_use]
extern crate notion_fail_derive;
use serde_json;

#[macro_export]
macro_rules! ok_or_panic {
    { $e:expr } => {
        match $e {
            Ok(x) => x,
            Err(err) => panic!("{} failed with {}", stringify!($e), err),
        }
    };
}

pub mod matchers;
pub mod paths;
pub mod process;
