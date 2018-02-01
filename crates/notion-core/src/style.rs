use std::convert::Into;

use console::style;
use failure;

pub fn display_error<E: Into<failure::Error>>(err: E) {
    display_error_prefix();
    eprintln!("{}", err.into());
}

pub fn display_error_prefix() {
    eprint!("{} ", style("error:").red().bold());
}
