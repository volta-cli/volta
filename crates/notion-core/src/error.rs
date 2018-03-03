//! Provides a protocol for Notion's error handling, including a subtrait of the `failure`
//! crate's `Fail` trait that manages the distinction between user-facing and internal
//! error messages, as well as the interface between errors and process exit codes.

use std::convert::{From, Into};
use std::fmt::{self, Display};

use failure::{self, Fail, Backtrace};

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

    /// Indicates the process exit code that should be returned if the process exits with this error.
    fn exit_code(&self) -> i32;
}

/// The `NotionError` type, which can contain any Notion failure.
#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub struct NotionError {
    error: failure::Error,
    user_friendly: bool,
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

    pub fn is_user_friendly(&self) -> bool { self.user_friendly }
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

pub trait ResultExt<T> {
    fn unknown(self) -> Result<T, NotionError>;
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

impl<T, E: Into<failure::Error>> ResultExt<T> for Result<T, E> {
    fn unknown(self) -> Result<T, NotionError> {
        self.map_err(|err| {
            UnknownNotionError { error: err.into() }.into()
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

pub type Fallible<T> = Result<T, NotionError>;
