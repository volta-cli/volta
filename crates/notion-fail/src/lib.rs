//! This crate provides a protocol for Notion's error handling, including a subtrait
//! of the [`failure`](https://github.com/rust-lang-nursery/failure) crate's
//! [`Fail`](https://docs.rs/failure/0.1.1/failure/trait.Fail.html) trait to manage
//! the distinction between user-facing and internal error messages, as well as
//! the interface between errors and process exit codes.
//!
//! # The `NotionFail` trait
//!
//! The main interface for Notion errors is `NotionFail`, which extends the
//! [`Fail`](https://docs.rs/failure/0.1.1/failure/trait.Fail.html) trait from the
//! [`failure`](https://github.com/rust-lang-nursery/failure) library with two additional
//! methods.
//!
//! ## User-friendly errors
//!
//! The `NotionFail::is_user_friendly()` method determines whether an error type is
//! intended for being presented to the end-user. The top-level logic of Notion uses
//! this to create a single catch-all behavior to present any non-user-friendly errors
//! as an internal error.
//!
//! ## Exit codes
//!
//! The `NotionFail::exit_code()` method allows each error type to indicate what the
//! process exit code should be if the error is the reason for exiting Notion.
//!
//! # The `NotionError` type and `Fallible` functions
//!
//! The main error type provided by this crate is `NotionError`. This acts more
//! or less as the "root" error type for Notion; all Notion error types can be
//! coerced into this type.
//!
//! If you don't have any need for more specific static information about the errors
//! that can be produced by a function, you should define its signature to return
//! `Result<T, NotionError>` (where `T` is whatever type you want for successful
//! results of the function).
//!
//! This is so common that you can use `Fallible<T>` as a shorthand.
//!
//! ## Example
//!
//! As a running example, we'll build a little parser for hex-encoded RGB triples.
//! The type could be defined as a struct of three bytes:
//!
//! ```
//! #[derive(Debug)]
//! struct Rgb { r: u8, g: u8, b: u8 }
//! ```
//!
//! A function that decodes a single two-digit component could then use `Fallible`
//! for its signature:
//!
//! ```
//! # #[macro_use]
//! # extern crate notion_fail;
//! #
//! use notion_fail::Fallible;
//! #
//! # #[derive(Debug)]
//! # struct Rgb { r: u8, g: u8, b: u8 }
//!
//! // same as: fn parse_component(src: &str, i: usize) -> Result<u8, NotionError>
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     // ...
//! #    Ok(17)
//! }
//! ```
//!
//! # Creating custom error types
//!
//! To create an error type in Notion, add a `#[derive]` attribute to derive the `Fail`
//! trait before the type declaration, and add a `#[fail(display = "...")]` attribute to
//! construct the error message string.
//!
//! If the error type is one that contains a user-friendly error message, declare an
//! implementation of `NotionFail` for the type where `is_user_friendly` returns `true`
//! and `exit_code` returns the process exit code for errors of this type.
//!
//! Continuing with the running example, we could create an error type for running past
//! the end of the input string:
//!
//! ## Example
//!
//! ```
//! # #[macro_use]
//! # extern crate notion_fail;
//! // required for `#[derive(Fail)]` and `#[fail(...)]` attributes
//! #[macro_use]
//! extern crate failure_derive;
//!
//! # extern crate failure;
//! use notion_fail::NotionFail;
//!
//! #[derive(Fail, Debug)]
//! #[fail(display = "unexpected end of string")]
//! struct UnexpectedEndOfString;
//!
//! impl NotionFail for UnexpectedEndOfString {
//!     // this is a user-friendly error type
//!     fn is_user_friendly(&self) -> bool { true }
//!
//!     // abort the process with exit code 101 if this failure goes uncaught
//!     fn exit_code(&self) -> i32 { 101 }
//! }
//! ```
//!
//! # Throwing errors
//!
//! The `throw!` macro is a convenient syntax for an early exit with an error. It
//! can be used inside any function with a `Result` return type (often a `Fallible<T>`).
//! The argument expression can evaluate to any type that implements a coercion to
//! the declared error type.
//!
//! ## Example
//!
//! ```
//! # #[macro_use]
//! # extern crate notion_fail;
//! # #[macro_use]
//! # extern crate failure_derive;
//! # extern crate failure;
//! # use notion_fail::{NotionFail, Fallible};
//! # #[derive(Fail, Debug)]
//! # #[fail(display = "unexpected end of string")]
//! # struct UnexpectedEndOfString;
//! #
//! # impl NotionFail for UnexpectedEndOfString {
//! #     // this is a user-friendly error type
//! #     fn is_user_friendly(&self) -> bool { true }
//! #
//! #     // abort the process with exit code 101 if this failure goes uncaught
//! #     fn exit_code(&self) -> i32 { 101 }
//! # }
//! #
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     if i + 2 > src.len() {
//!         // UnexpectedEndOfString implements NotionFail, so it coerces to NotionError
//!         throw!(UnexpectedEndOfString);
//!     }
//!
//!     // ...
//! #   Ok(0)
//! }
//! ```
//!
//! # Using third-party error types
//!
//! When using a third-party library that has error types of its own, those error types
//! need to be converted to Notion errors. Since third party libraries have not been
//! designed with Notion's end-user error messages in mind, third-party error types are
//! not automatically converted into Notion errors.
//!
//! Instead, this crate provides a couple of extension traits that you can import to
//! add an `unknown()` method to errors (`FailExt`) or `Result`s (`ResultExt`). This
//! method will convert any third-party error to a Notion error. The resulting Notion
//! error will be treated as an internal error. (But see the sections below to learn
//! how to wrap internal errors with user-friendly messages without losing data.)
//!
//! ## Example
//!
//! ```
//! # #[macro_use]
//! # extern crate notion_fail;
//! # #[macro_use]
//! # extern crate failure_derive;
//! # extern crate failure;
//! # use notion_fail::{NotionFail, Fallible};
//! // add `unknown()` extension method to Results
//! use notion_fail::ResultExt;
//! # #[derive(Fail, Debug)]
//! # #[fail(display = "unexpected end of string")]
//! # struct UnexpectedEndOfString;
//! #
//! # impl NotionFail for UnexpectedEndOfString {
//! #     // this is a user-friendly error type
//! #     fn is_user_friendly(&self) -> bool { true }
//! #
//! #     // abort the process with exit code 101 if this failure goes uncaught
//! #     fn exit_code(&self) -> i32 { 101 }
//! # }
//!
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     if i + 2 > src.len() {
//!         // UnexpectedEndOfString implements NotionFail, so it coerces to NotionError
//!         throw!(UnexpectedEndOfString);
//!     }
//!
//!     // convert the std::num::ParseIntError into a NotionError
//!     u8::from_str_radix(&src[i..i + 2], 16).unknown()
//! }
//! ```
//!
//! # Cause chains
//!
//! Since errors get propagated up from lower abstraction layers to higher ones, the
//! higher layers of abstraction often need to add contextual information to the error
//! messages, producing higher quality messages.
//!
//! For example, the `ParseIntError` produced by `u8::from_str_radix` does not tell
//! the end user that we were parsing an integer in the context of parsing an RGB
//! value.
//!
//! To add contextual information to a lower layer's error, we use the `with_context`
//! method and pass it a closure that takes a reference to the lower layer's error
//! and uses it to construct a new higher-level error.
//!
//! A powerful feature of `with_context` is that it saves the lower-level
//! error message as part of a _cause_ chain, which Notion's top-level can then use
//! to produce in-depth diagnostics in a log file or for `--verbose` error reporting.
//! Most error handling logic should not need to work with cause chains, so this is
//! all handled automatically.
//!
//! ## Example
//!
//! ```
//! # #[macro_use]
//! # extern crate notion_fail;
//! # #[macro_use]
//! # extern crate failure_derive;
//! # extern crate failure;
//! # use notion_fail::{NotionFail, Fallible};
//! // add `unknown()` and `with_context()` extension methosd to Results
//! use notion_fail::ResultExt;
//! # use std::fmt::Display;
//!
//! # #[derive(Fail, Debug)]
//! # #[fail(display = "unexpected end of string")]
//! # struct UnexpectedEndOfString;
//! #
//! # impl NotionFail for UnexpectedEndOfString {
//! #     // this is a user-friendly error type
//! #     fn is_user_friendly(&self) -> bool { true }
//! #
//! #     // abort the process with exit code 101 if this failure goes uncaught
//! #     fn exit_code(&self) -> i32 { 101 }
//! # }
//! #
//! # fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//! #     if i + 2 > src.len() {
//! #         // UnexpectedEndOfString implements NotionFail, so it coerces to NotionError
//! #         throw!(UnexpectedEndOfString);
//! #     }
//! #
//! #     // convert the std::num::ParseIntError into a NotionError
//! #     u8::from_str_radix(&src[i..i + 2], 16).unknown()
//! # }
//! #[derive(Fail, Debug)]
//! #[fail(display = "invalid RGB string: ", details)]
//! struct InvalidRgbString { details: String }
//!
//! impl NotionFail for InvalidRgbString {
//!     fn is_user_friendly(&self) -> bool { true}
//!     fn exit_code(&self) -> i32 { 101 }
//! }
//!
//! impl InvalidRgbString {
//!     fn new<D: Display>(details: &D) -> InvalidRgbString {
//!         InvalidRgbString { details: format!("{}", details) }
//!     }
//! }
//!
//! impl Rgb {
//!     fn parse(src: &str) -> Fallible<Rgb> {
//!         Ok(Rgb {
//!             r: parse_component(src, 0).with_context(InvalidRgbString::new)?,
//!             g: parse_component(src, 2).with_context(InvalidRgbString::new)?,
//!             b: parse_component(src, 4).with_context(InvalidRgbString::new)?
//!         })
//!     }
//! }
//! ```
//!
//! Notice that you can use `with_context` to wrap any kind of error, including
//! errors that may already be user-friendly. So you can always use this to add
//! even more clarity to any errors. For instance, in our running example of an
//! RGB parser, a higher layer may want to add context about _which_ RGB string
//! was being parsed and where it came from (say, the filename and line number).

extern crate failure;
#[macro_use]
extern crate failure_derive;

use std::convert::{From, Into};
use std::fmt::{self, Display};

use failure::{Fail, Backtrace};

/// A temporary polyfill for `throw!` until the new `failure` library includes it.
#[macro_export]
macro_rules! throw {
    ($e:expr) => {
        return Err(::std::convert::Into::into($e));
    }
}

/// The failure trait for all Notion errors.
pub trait NotionFail: Fail {
    /// Indicates whether this error has a message suitable for reporting to an end-user.
    fn is_user_friendly(&self) -> bool;

    /// Returns the process exit code that should be returned if the process exits with this error.
    fn exit_code(&self) -> i32;
}

/// The `NotionError` type, which can contain any Notion failure.
#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub struct NotionError {
    /// The underlying error.
    error: failure::Error,

    /// The result of `error.is_user_friendly()`.
    user_friendly: bool,

    /// The result of `error.exit_code()`.
    exit_code: i32
}

impl NotionError {
    /// Returns a reference to the underlying failure of this error.
    pub fn as_fail(&self) -> &Fail {
        self.error.cause()
    }

    /// Gets a reference to the `Backtrace` for this error.
    pub fn backtrace(&self) -> &Backtrace {
        self.error.backtrace()
    }

    /// Attempts to downcast this error to a particular `NotionFail` type by reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_ref<T: NotionFail>(&self) -> Option<&T> {
        self.error.downcast_ref()
    }

    /// Attempts to downcast this error to a particular `NotionFail` type by mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: NotionFail>(&mut self) -> Option<&mut T> {
        self.error.downcast_mut()
    }

    /// Indicates whether this error has a message suitable for reporting to an end-user.
    pub fn is_user_friendly(&self) -> bool { self.user_friendly }

    /// Returns the process exit code that should be returned if the process exits with this error.
    pub fn exit_code(&self) -> i32 { self.exit_code }
}

impl<T: NotionFail> From<T> for NotionError {
    fn from(failure: T) -> Self {
        let user_friendly = failure.is_user_friendly();
        let exit_code = failure.exit_code();
        NotionError {
            error: failure.into(),
            user_friendly,
            exit_code
        }
    }
}

/// An extension trait allowing any failure, including failures from external libraries,
/// to be converted to a Notion error. This marks the error as an unknown error, i.e.
/// a non-user-friendly error.
pub trait FailExt {
    fn unknown(self) -> NotionError;
    fn with_context<F, D>(self, f: F) -> NotionError
        where F: FnOnce(&Self) -> D,
              D: NotionFail;
}

/// An extension trait for `Result` values, allowing conversion of third-party errors
/// or other lower-layer errors into Notion errors.
pub trait ResultExt<T, E> {
    /// Convert any error-producing result into a `NotionError`-producing result.
    fn unknown(self) -> Result<T, NotionError>;

    /// Wrap any error-producing result in a higher-layer error-producing result, pushing
    /// the lower-layer error onto the cause chain.
    fn with_context<F, D>(self, f: F) -> Result<T, NotionError>
        where F: FnOnce(&E) -> D,
              D: NotionFail;
}

/// A wrapper type for unknown errors.
#[derive(Debug)]
struct UnknownNotionError {
    error: failure::Error
}

impl Display for UnknownNotionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An unknown error has occurred")
    }
}

impl Fail for UnknownNotionError {
    fn cause(&self) -> Option<&Fail> {
        Some(self.error.cause())
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }
}

impl NotionFail for UnknownNotionError {
    fn is_user_friendly(&self) -> bool { false }
    fn exit_code(&self) -> i32 { 1 }
}

impl<E: Into<failure::Error>> FailExt for E {
    fn unknown(self) -> NotionError {
        UnknownNotionError { error: self.into() }.into()
    }

    fn with_context<F, D>(self, f: F) -> NotionError
        where F: FnOnce(&Self) -> D,
              D: NotionFail
    {
        let display = f(&self);
        let error: failure::Error = self.into();
        let context = error.context(display);
        context.into()
    }
}

impl<T, E: Into<failure::Error>> ResultExt<T, E> for Result<T, E> {
    fn unknown(self) -> Result<T, NotionError> {
        self.map_err(|err| {
            UnknownNotionError { error: err.into() }.into()
        })
    }

    fn with_context<F, D>(self, f: F) -> Result<T, NotionError>
        where F: FnOnce(&E) -> D,
              D: NotionFail
    {
        self.map_err(|err| {
            err.with_context(f)
        })
    }
}

impl<D: NotionFail> NotionFail for failure::Context<D> {
    fn is_user_friendly(&self) -> bool {
        self.get_context().is_user_friendly()
    }

    fn exit_code(&self) -> i32 {
        self.get_context().exit_code()
    }
}

/// A convenient shorthand for `Result` types that produce `NotionError`s.
pub type Fallible<T> = Result<T, NotionError>;
