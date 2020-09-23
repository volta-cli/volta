use std::fmt;
use std::process::exit;
use std::{error::Error, io};

mod kind;
mod reporter;

pub use kind::ErrorKind;
pub use reporter::report_error;

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

    /// Verifies if the inner source is `io::ErrorKind::NotFound`
    pub fn is_not_found_error_kind(&self) -> bool {
        self.source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .and_then(|io_err| {
                if io_err.kind() == io::ErrorKind::NotFound {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false)
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
    /// Transforms an `Fallible<T>::Err(x)` into `Fallible<T>::Ok(T::default())`
    ///
    ///# Example
    ///```
    ///use std::io;
    ///use io::Error;
    ///use volta_core::error::{Fallible, VoltaError, ErrorKind};
    ///use crate::volta_core::error::AcceptableErrorToDefault;
    ///
    ///let not_found = Error::new(io::ErrorKind::NotFound, "file not found");
    ///let volta_error = VoltaError::from_source(not_found, ErrorKind::BinaryExecError);
    ///let fallible: Fallible<i32> = Err(volta_error);
    ///if let Ok(zero) = fallible.accept_error_as_default_if(|e| e.is_not_found_error_kind()) {
    ///    assert_eq!(zero, 0);
    ///} else {
    ///    assert!(false, "accept_error_as_value_if is not equal to 10");
    ///}
    ///```
    fn accept_error_as_default_if<F>(self, accept: F) -> Fallible<T>
    where
        F: FnOnce(&VoltaError) -> bool,
    {
        self.accept_error_as_value_if(accept, || T::default())
    }
}

/// Extension to `Fallible<T>` to transform a `VoltaError` into a value
impl<T> AcceptableErrorToValue<T> for Fallible<T> {
    ///
    ///# Example
    ///```
    ///use std::io;
    ///use io::Error;
    ///use volta_core::error::{Fallible, VoltaError, ErrorKind};
    ///use crate::volta_core::error::AcceptableErrorToValue;
    ///
    ///let not_found = Error::new(io::ErrorKind::NotFound, "file not found");
    ///let volta_error = VoltaError::from_source(not_found, ErrorKind::BinaryExecError);
    ///let fallible: Fallible<i32> = Err(volta_error);
    ///if let Ok(ten) = fallible.accept_error_as_value_if(|e| e.is_not_found_error_kind(), || 10) {
    ///    assert_eq!(ten, 10);
    ///} else {
    ///    assert!(false, "accept_error_as_value_if is not equal to 10");
    ///}
    ///```
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
