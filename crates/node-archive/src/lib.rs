//! This crate provides types for fetching and unpacking Node distribution
//! archives, which is a tarball for Unixes and a zipfile for Windows.
//!
//! These docs show the top-level exports of this crate as re-exported of
//! the `tarball` module (due to limitations of rustdoc); the top-level
//! exports are re-exported from `tarball` on Unix operating systems and
//! from `zip` on Windows operating systems.

#![cfg_attr(feature = "universal-docs", feature(doc_cfg))]

// This has to be included here before the `tarball` and `zip` modules are added,
// so that they can use the `define_source_trait!` macro.
#[macro_use]
mod source;

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "universal-docs")] {
        extern crate tar;
        extern crate flate2;

        #[doc(cfg(unix))]
        mod tarball;

        extern crate zip as zip_rs;
        extern crate verbatim;

        #[doc(cfg(windows))]
        mod zip;
    } else if #[cfg(unix)] {
        extern crate tar;
        extern crate flate2;

        mod tarball;
    } else if #[cfg(windows)] {
        extern crate zip as zip_rs;
        extern crate verbatim;

        mod zip;
    } else {
        compile_error!("Unsupported OS (expected 'unix' or 'windows').");
    }
}

extern crate reqwest;
extern crate tee;
extern crate progress_read;

#[macro_use]
extern crate failure_derive;
extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "HTTP failure ({})", code)]
pub(crate) struct HttpError {
    code: ::reqwest::StatusCode
}

cfg_if! {
    if #[cfg(unix)] {
        pub use tarball::{Archive, Cached, Remote, Source};
    } else if #[cfg(windows)] {
        pub use zip::{Archive, Cached, Remote, Source};
    } else {
        compile_error!("Unsupported OS (expected 'unix' or 'windows').");
    }
}

impl Source for Box<Source> {
    fn uncompressed_size(&self) -> Option<u64> {
        (**self).uncompressed_size()
    }

    fn compressed_size(&self) -> u64 {
        (**self).compressed_size()
    }
}
