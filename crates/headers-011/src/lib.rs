use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::hyper_011::header::Header;

/// A compatibility trait for `reqwest` 0.9.x to support an API for extracting
/// typed headers similar to the `reqwest` <= 0.8.x API, using the `hyper-011`
/// bridge that can be enabled in `reqwest` with the `"hyper-011"` feature.
///
/// According to the
/// [changelog](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md#breaking-changes),
/// the removal of the typed headers API is temporary. Once that functionality
/// is added back to the core of `reqwest`, this crate should become
/// unnecessary.
pub trait Headers011 {
    fn get_raw(&self, key: &str) -> Option<&HeaderValue>;

    /// Extract a typed header.
    fn get_011<H: Header>(&self) -> Option<H> {
        self
            .get_raw(H::header_name())
            .and_then(|value| {
                H::parse_header(&value.as_bytes().into()).ok()
            })
    }
}

impl Headers011 for HeaderMap {
    fn get_raw(&self, key: &str) -> Option<&HeaderValue> { self.get(key) }
}
