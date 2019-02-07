//! Utilities to use with acceptance tests in Notion.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

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
