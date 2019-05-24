//! This module provides a custom Logger implementation for use with the `log` crate
use atty::Stream;
use console::style;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use std::env;
use std::fmt::Display;
use textwrap::{NoHyphenation, Wrapper};

use crate::style::text_width;

const ERROR_PREFIX: &'static str = "error:";
const WARNING_PREFIX: &'static str = "warning:";
const SHIM_ERROR_PREFIX: &'static str = "Volta error:";
const SHIM_WARNING_PREFIX: &'static str = "Volta warning:";
const VOLTA_DEV: &'static str = "VOLTA_DEV";
const ALLOWED_CRATE: &'static str = "volta_core";
const WRAP_INDENT: &'static str = "    ";

/// Represents the context from which the logger was created
pub enum LogContext {
    /// Log messages from the `volta` executable
    Volta,

    /// Log messages from one of the shims
    Shim,
}

/// Represents the level of verbosity that was requested by the user
pub enum LogVerbosity {
    Quiet,
    Default,
    Verbose,
}

pub struct Logger {
    context: LogContext,
    level: Level,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && record.target().starts_with(ALLOWED_CRATE) {
            match record.level() {
                Level::Error => self.log_error(record.args()),
                Level::Warn => self.log_warning(record.args()),
                _ => println!("{}", record.args()),
            }
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Initialize the global logger with a Logger instance
    /// Will use the requested level of Verbosity
    /// If set to Default, will use the environment to determine the level of verbosity
    pub fn init(context: LogContext, verbosity: LogVerbosity) -> Result<(), SetLoggerError> {
        let logger = Logger::new(context, verbosity);
        log::set_max_level(logger.level.to_level_filter());
        log::set_boxed_logger(Box::new(logger))?;
        Ok(())
    }

    fn new(context: LogContext, verbosity: LogVerbosity) -> Self {
        let level = match verbosity {
            LogVerbosity::Quiet => Level::Error,
            LogVerbosity::Default => level_from_env(),
            LogVerbosity::Verbose => Level::Debug,
        };

        Logger { context, level }
    }

    fn log_error<D>(&self, message: &D)
    where
        D: Display,
    {
        let prefix = match &self.context {
            LogContext::Volta => ERROR_PREFIX,
            LogContext::Shim => SHIM_ERROR_PREFIX,
        };

        eprintln!("{} {}", style(prefix).red().bold(), message);
    }

    fn log_warning<D>(&self, message: &D)
    where
        D: Display,
    {
        let prefix = match &self.context {
            LogContext::Volta => WARNING_PREFIX,
            LogContext::Shim => SHIM_WARNING_PREFIX,
        };

        println!(
            "{}{}",
            style(prefix).yellow().bold(),
            wrap_content(prefix, message)
        );
    }
}

/// Wraps the supplied content to the terminal width, if we are in a terminal.
/// If not, returns the content as a String
///
/// Note: Uses the supplied prefix to calculate the terminal width, but then removes
/// it so that it can be styled (style characters are counted against the wrapped width)
fn wrap_content<D>(prefix: &str, content: &D) -> String
where
    D: Display,
{
    match text_width() {
        Some(width) => Wrapper::with_splitter(width, NoHyphenation)
            .subsequent_indent(WRAP_INDENT)
            .break_words(false)
            .fill(&format!("{} {}", prefix, content))
            .replace(prefix, ""),
        None => format!(" {}", content),
    }
}

/// Determines the correct logging level based on the environment
/// If VOLTA_DEV is set then we use Debug, which is the same as setting --verbose
/// If not, we check the current stdout for whether it is a TTY
///     If it is a TTY, we use Info
///     If it is NOT a TTY, we use Error as we don't want to show warnings when running as a script
fn level_from_env() -> Level {
    match (env::var(VOLTA_DEV), atty::is(Stream::Stdout)) {
        (Ok(_), _) => Level::Debug,
        (_, true) => Level::Info,
        (_, false) => Level::Error,
    }
}

#[cfg(test)]
mod tests {}
