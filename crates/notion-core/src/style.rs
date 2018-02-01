use std::convert::Into;

use console::style;
use failure;

pub fn display_error<E: Into<failure::Error>>(err: E) {
    eprintln!("{} {}", style("error:").red().bold(), err.into());
}
