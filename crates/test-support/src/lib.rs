//! Utilities to use with acceptance tests in Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

extern crate hamcrest2;
extern crate serde_json;

pub mod matchers;
pub mod process;

extern crate failure;
extern crate failure_derive;
#[macro_use]
extern crate notion_fail;
#[macro_use]
extern crate notion_fail_derive;
