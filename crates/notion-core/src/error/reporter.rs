use std::env;
use std::io::Result;
use std::path::PathBuf;

use crate::style::{format_error_details, format_error_message};
use notion_fail::NotionError;

const NOTION_DEV: &'static str = "NOTION_DEV";

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `notion` executable.
    Notion,

    /// An error reported from a shim.
    Shim,
}

pub enum ErrorReporter {
    /// Reports errors in the standard, concise format
    Standard,

    /// Reports errors with additional details
    Verbose,
}

impl ErrorReporter {
    /// Create a new ErrorReporter of the default type
    pub fn new() -> Self {
        if env::var(NOTION_DEV).is_ok() {
            ErrorReporter::Verbose
        } else {
            ErrorReporter::Standard
        }
    }

    /// Create a new verbose ErrorReporter
    pub fn verbose() -> Self {
        ErrorReporter::Verbose
    }

    /// Report an error, both to the terminal and the error log
    pub fn report(&self, cx: ErrorContext, err: &NotionError) {
        let message = format_error_message(cx, err);
        let details = format_error_details(err);

        match self {
            ErrorReporter::Standard => eprint!("{}", message),
            ErrorReporter::Verbose => eprint!("{}{}", message, details),
        }

        match write_error_log(message, details) {
            Ok(_log_file) => {}
            Err(_) => {}
        }
    }
}

fn write_error_log(_message: String, _details: String) -> Result<PathBuf> {
    Ok(PathBuf::from(""))
}
