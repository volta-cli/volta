//! The view layer of Volta, with utilities for styling command-line output.
use archive::Origin;
use atty::Stream;
use console::{style, StyledObject};
use failure::Fail;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;
use textwrap::{NoHyphenation, Wrapper};

use volta_fail::{Fallible, VoltaError};

use crate::error::{ErrorContext, ErrorDetails};

// ISSUE #306 - When unknown error messages are removed, this can be removed as well
const INTERNAL_ERROR_MESSAGE: &'static str = "an internal error occurred

Volta is still a pre-alpha project, so we expect to run into some bugs,
but we'd love to hear about them so we can fix them!

Please feel free to reach out to us at \x1b[36m\x1b[1m@voltajs\x1b[0m on Twitter or file an issue at:

    \x1b[1mhttps://github.com/volta-cli/volta/issues\x1b[0m
";

const WARNING_PREFIX: &'static str = "warning:";
const SHIM_WARNING_PREFIX: &'static str = "Volta warning:";

/// Generate the styled prefix for a success message
pub(crate) fn success_prefix() -> StyledObject<&'static str> {
    style("success:").green().bold()
}

fn styled_warning_prefix(prefix: &'static str) -> StyledObject<&'static str> {
    style(prefix).yellow().bold()
}

enum Context {
    Volta,
    Shim,
}

/// Determine whether we're in a shim context or a Volta context.
fn context() -> Fallible<Context> {
    let command = std::env::args_os()
        .next()
        .map(|os_str| std::path::PathBuf::from(os_str));

    let command = command.and_then(|p| {
        p.file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
    });

    let command = command.as_ref().map(String::as_str);

    match command {
        Some("volta") => Ok(Context::Volta),
        Some(_) => Ok(Context::Shim),
        None => Err(ErrorDetails::CouldNotDetermineTool.into()),
    }
}

pub(crate) fn write_warning(message: &str) -> Fallible<()> {
    // If we're not in a tty, don't write warnings as they could mess up scripts
    if atty::isnt(Stream::Stdout) {
        return Ok(());
    }

    let prefix = match context()? {
        Context::Volta => WARNING_PREFIX,
        Context::Shim => SHIM_WARNING_PREFIX,
    };

    // We're use the `NoHyphenation` splitter mode so that paths with `-`
    // characters don't trigger wrapping.
    let wrapped = Wrapper::with_splitter(text_width(), NoHyphenation)
        .subsequent_indent("    ")
        .break_words(false)
        .fill(&format!("{} {}", prefix, message))
        .replace(prefix, "");

    println!("{}{}", styled_warning_prefix(prefix), wrapped);

    Ok(())
}

/// Format an error for output in the given context
pub(crate) fn format_error_message(cx: ErrorContext, err: &VoltaError) -> String {
    let prefix = error_prefix(cx);

    if err.is_user_friendly() {
        format!("{} {}", prefix, err)
    } else {
        format!("{} {}", prefix, INTERNAL_ERROR_MESSAGE)
    }
}

/// Format the underlying cause of an error
pub(crate) fn format_error_cause(inner: &Fail) -> String {
    format!(
        "{}{} {}",
        style("cause").underlined().bold(),
        style(":").bold(),
        inner
    )
}

fn error_prefix(cx: ErrorContext) -> StyledObject<&'static str> {
    match cx {
        ErrorContext::Volta => {
            // Since the command here was `volta`, it would be redundant to say that this was
            // a Volta error, so we are less explicit in the heading.
            style("error:").red().bold()
        }
        ErrorContext::Shim => {
            // Since a Volta error is rare case for a shim, it can be surprising to a user.
            // To make it extra clear that this was a failure that happened in Volta when
            // attempting to delegate to a shim, we are more explicit about the fact that it's
            // a Volta error.
            style("Volta error:").red().bold()
        }
    }
}

/// Determines the string to display based on the Origin of the operation.
fn action_str(origin: Origin) -> &'static str {
    match origin {
        Origin::Local => "Unpacking",
        Origin::Remote => "Fetching",
    }
}

pub fn tool_version<N, V>(name: N, version: V) -> String
where
    N: std::fmt::Display + Sized,
    V: std::fmt::Display + Sized,
{
    format!("{:}@{:}", name, version)
}

const MAX_WIDTH: usize = 100;

/// Get the display width. If it is unavailable, supply a normal default.
pub fn display_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(MAX_WIDTH)
}

pub fn text_width() -> usize {
    // We want the lesser of our preferred max and the user's term size.
    display_width().min(MAX_WIDTH)
}

/// Constructs a command-line progress bar based on the specified Origin enum
/// (e.g., `Origin::Remote`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(origin: Origin, details: &str, len: u64) -> ProgressBar {
    let action = action_str(origin);
    let action_width = action.len() + 2; // plus 2 spaces to look nice
    let msg_width = action_width + 1 + details.len();

    //   Installing v1.23.4  [====================>                   ]  50%
    // |----------| |-----|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let available_width = display_width() - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(&format!(
        "{: >width$} {}",
        style(action).green().bold(),
        details,
        width = action_width,
    ));
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%",
                bar_width
            ))
            .progress_chars("=> "),
    );

    bar
}

/// Constructs a command-line progress spinner with the specified "message"
/// string. The spinner is ticked by default every 20ms.
pub fn progress_spinner(message: &str) -> ProgressBar {
    // ⠋ Fetching public registry: https://nodejs.org/dist/index.json
    let spinner = ProgressBar::new_spinner();

    spinner.set_message(message);
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}"));
    spinner.enable_steady_tick(20); // tick the spinner every 20ms

    spinner
}
