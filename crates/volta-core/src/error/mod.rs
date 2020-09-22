use std::fmt;
use std::process::exit;
use std::{error::Error, io};

mod kind;
mod reporter;

pub use kind::ErrorKind;
pub use reporter::report_error;

use crate::fs::is_not_found_error_kind;

pub type Fallible<T> = Result<T, VoltaError>;

/// Error type for Volta
#[derive(Debug)]
pub struct VoltaError {
    inner: Box<Inner>,
}

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    source: Option<Box<dyn Error>>,
}

impl VoltaError {
    /// The exit code Volta should use when this error stops execution
    pub fn exit_code(&self) -> ExitCode {
        self.inner.kind.exit_code()
    }

    /// Create a new VoltaError instance including a source error
    pub fn from_source<E>(source: E, kind: ErrorKind) -> Self
    where
        E: Into<Box<dyn Error>>,
    {
        VoltaError {
            inner: Box::new(Inner {
                kind,
                source: Some(source.into()),
            }),
        }
    }

    /// Get a reference to the ErrorKind for this error
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }

    /// TODO:
    /// 1. find a good name
    /// 2. determine if it can be removed
    pub fn not_found_to_ok<T>(self, to: T) -> Fallible<T> {
        self.source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .and_then(|io_err| {
                if io_err.kind() == io::ErrorKind::NotFound {
                    Some(to)
                } else {
                    None
                }
            })
            .ok_or(self)
    }

    /// TODO: find a good name
    pub fn is_io_not_found(&self) -> bool {
        is_not_found_error_kind(self)
    }
}

impl fmt::Display for VoltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.kind.fmt(f)
    }
}

impl Error for VoltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source.as_ref().map(|b| b.as_ref())
    }
}

impl From<ErrorKind> for VoltaError {
    fn from(kind: ErrorKind) -> Self {
        VoltaError {
            inner: Box::new(Inner { kind, source: None }),
        }
    }
}

pub trait AcceptableErrorToValue<T> {
    fn accept_error_as_value_if<F1, F2>(self, accept_if: F1, as_value: F2) -> Fallible<T>
    where
        F1: FnOnce(&VoltaError) -> bool,
        F2: FnOnce() -> T;
}

pub trait AcceptableErrorToDefault<T>: AcceptableErrorToValue<T>
where
    T: Default,
    Self: Sized,
{
    fn error_to_default_if<F>(self, accept: F) -> Fallible<T>
    where
        F: FnOnce(&VoltaError) -> bool,
    {
        self.accept_error_as_value_if(accept, || T::default())
    }
}

impl<T> AcceptableErrorToValue<T> for Fallible<T> {
    fn accept_error_as_value_if<F1, F2>(self, accept_if: F1, as_value: F2) -> Fallible<T>
    where
        F1: FnOnce(&VoltaError) -> bool,
        F2: FnOnce() -> T,
    {
        match self {
            Err(error) => {
                if accept_if(&error) {
                    Ok(as_value())
                } else {
                    Err(error)
                }
            }
            Ok(v) => Ok(v),
        }
    }
}

impl<T> AcceptableErrorToDefault<T> for Fallible<T> where T: Default {}

/// Trait providing the with_context method to easily convert any Result error into a VoltaError
pub trait Context<T> {
    fn with_context<F>(self, f: F) -> Fallible<T>
    where
        F: FnOnce() -> ErrorKind;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Error + 'static,
{
    fn with_context<F>(self, f: F) -> Fallible<T>
    where
        F: FnOnce() -> ErrorKind,
    {
        self.map_err(|e| VoltaError::from_source(e, f()))
    }
}

/// Exit codes supported by Volta Errors
#[derive(Copy, Clone, Debug)]
pub enum ExitCode {
    /// No error occurred.
    Success = 0,

    /// An unknown error occurred.
    UnknownError = 1,

    /// An invalid combination of command-line arguments was supplied.
    InvalidArguments = 3,

    /// No match could be found for the requested version string.
    NoVersionMatch = 4,

    /// A network error occurred.
    NetworkError = 5,

    /// A required environment variable was unset or invalid.
    EnvironmentError = 6,

    /// A file could not be read or written.
    FileSystemError = 7,

    /// Package configuration is missing or incorrect.
    ConfigurationError = 8,

    /// The command or feature is not yet implemented.
    NotYetImplemented = 9,

    /// The requested executable could not be run.
    ExecutionFailure = 126,

    /// The requested executable is not available.
    ExecutableNotFound = 127,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        exit(self as i32);
    }
}
