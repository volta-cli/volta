use docopt;
use failure::Context;
use notion_core::error::ErrorDetails;
use notion_fail::NotionError;

pub(crate) fn cli_parse_error(error: &docopt::Error) -> ErrorDetails {
    if let &docopt::Error::WithProgramUsage(ref real_error, ref usage) = error {
        ErrorDetails::CliParseError {
            usage: Some(usage.clone()),
            error: real_error.to_string(),
        }
    } else {
        ErrorDetails::CliParseError {
            usage: None,
            error: error.to_string(),
        }
    }
}

pub(crate) trait DocoptExt {
    fn is_help(&self) -> bool;
    fn is_version(&self) -> bool;
}

impl DocoptExt for docopt::Error {
    fn is_help(&self) -> bool {
        match self {
            &docopt::Error::Help => true,
            &docopt::Error::WithProgramUsage(ref error, _) => error.is_help(),
            _ => false,
        }
    }

    fn is_version(&self) -> bool {
        match self {
            &docopt::Error::Version(_) => true,
            &docopt::Error::WithProgramUsage(ref error, _) => error.is_version(),
            _ => false,
        }
    }
}

pub(crate) trait NotionErrorExt {
    fn usage(&self) -> Option<&str>;
}

impl NotionErrorExt for NotionError {
    fn usage(&self) -> Option<&str> {
        if let Some(ctx) = self.as_fail().downcast_ref::<Context<ErrorDetails>>() {
            if let ErrorDetails::CliParseError {
                usage: Some(ref usage),
                ..
            } = ctx.get_context()
            {
                return Some(usage);
            }
        }
        None
    }
}
