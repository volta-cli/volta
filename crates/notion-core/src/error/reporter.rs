use std::env::{self, args_os};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;

use crate::fs::ensure_containing_dir_exists;
use crate::path::log_dir;
use crate::style::{format_error_cause, format_error_message};
use chrono::Local;
use failure::Error;
use notion_fail::NotionError;

const NOTION_DEV: &'static str = "NOTION_DEV";

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `notion` executable.
    Notion,

    /// An error reported from a shim.
    Shim,
}

/// Reporter for showing errors to the terminal and error logs
pub struct ErrorReporter {
    /// Notion version to display in error logs
    version: String,

    /// Flag indicating whether to report additional details to the terminal
    verbose: bool,
}

impl ErrorReporter {
    /// Create a new ErrorReporter from a verbose flag
    pub fn from_flag(notion_version: &str, verbose: bool) -> Self {
        if verbose {
            ErrorReporter {
                version: notion_version.to_string(),
                verbose,
            }
        } else {
            ErrorReporter::from_env(notion_version)
        }
    }

    /// Create a new ErrorReporter from the environment variables
    pub fn from_env(notion_version: &str) -> Self {
        ErrorReporter {
            version: notion_version.to_string(),
            verbose: env::var(NOTION_DEV).is_ok(),
        }
    }

    /// Report an error, both to the terminal and the error log
    pub fn report(&self, cx: ErrorContext, err: &NotionError) {
        let message = format_error_message(cx, err);

        eprintln!("{}", message);

        if let Some(details) = compose_error_details(err) {
            if self.verbose {
                eprintln!();
                eprintln!("{}", details);
            }

            match self.write_error_log(message, details) {
                Ok(log_file) => {
                    eprintln!("Error details written to: {}", log_file);
                }
                Err(_) => {
                    eprintln!("Unable to write error log!");
                }
            }
        }
    }

    /// Write an error log with additional details about the error
    fn write_error_log(&self, message: String, details: String) -> Result<String, Error> {
        let file_name = Local::now()
            .format("notion-error-%Y-%m-%d_%H_%M_%S%.3f.log")
            .to_string();
        let log_file_path = log_dir()?.join(&file_name);

        ensure_containing_dir_exists(&log_file_path)?;
        let mut log_file = File::create(log_file_path)?;

        writeln!(log_file, "{}", collect_arguments())?;
        writeln!(log_file, "Notion v{}", self.version)?;
        writeln!(log_file)?;
        writeln!(log_file, "{}", message)?;
        writeln!(log_file)?;
        writeln!(log_file, "{}", details)?;

        Ok(file_name)
    }
}

fn compose_error_details(err: &NotionError) -> Option<String> {
    // Only compose details if there is an underlying cause for the error
    let mut current = match err.as_fail().cause() {
        Some(cause) => cause,
        None => {
            return None;
        }
    };
    let mut details = String::new();

    // Walk up the tree of causes and include all of them
    loop {
        details.push_str(&format_error_cause(current));

        match current.cause() {
            Some(cause) => {
                details.push_str("\n\n");
                current = cause;
            }
            None => {
                break;
            }
        };
    }

    // ISSUE #75 - Once we have a way to determine backtraces without RUST_BACKTRACE, we can make this always available
    // Until then, we know that if the env var is not set, the backtrace will be empty
    if env::var("RUST_BACKTRACE").is_ok() {
        // Note: The implementation of `Display` for Backtrace includes a 'stack backtrace:' prefix
        write!(details, "\n\n{}", err.backtrace()).expect("write! to a String doesn't fail");
    }

    Some(details)
}

/// Combines all the arguments into a single String
fn collect_arguments() -> String {
    args_os()
        .map(|arg| arg.into_string().unwrap_or(String::from("<UNKNOWN>")))
        .collect::<Vec<String>>()
        .join(" ")
}
