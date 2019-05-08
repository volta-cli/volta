//! The view layer of Volta, with utilities for styling command-line output.
use crate::error::ErrorContext;
use archive::Origin;
use console::{style, StyledObject};
use failure::Fail;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;
use volta_fail::VoltaError;

// ISSUE #306 - When unknown error messages are removed, this can be removed as well
const INTERNAL_ERROR_MESSAGE: &'static str = "an internal error occurred

Volta is still a pre-alpha project, so we expect to run into some bugs,
but we'd love to hear about them so we can fix them!

Please feel free to reach out to us at \x1b[36m\x1b[1m@voltajs\x1b[0m on Twitter or file an issue at:

    \x1b[1mhttps://github.com/volta-cli/volta/issues\x1b[0m
";

/// Generate the styled prefix for a success message
pub(crate) fn success_prefix() -> StyledObject<&'static str> {
    style("success:").green().bold()
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

/// Get the display width. If it is unavailable, supply a normal default.
pub fn display_width() -> usize {
    term_size::dimensions().map(|(w, _)| w).unwrap_or(80)
}

pub fn text_width() -> usize {
    display_width().min(80)
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
