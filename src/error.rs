use docopt;
use failure::Context;
use notion_fail::{ExitCode, NotionError, NotionFail};

#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub(crate) struct CliParseError {
    pub(crate) usage: Option<String>,
    pub(crate) error: String,
}

impl CliParseError {
    pub(crate) fn from_docopt(error: &docopt::Error) -> Self {
        if let &docopt::Error::WithProgramUsage(ref real_error, ref usage) = error {
            CliParseError {
                usage: Some(usage.clone()),
                error: real_error.to_string(),
            }
        } else {
            CliParseError {
                usage: None,
                error: error.to_string(),
            }
        }
    }
}

impl NotionFail for CliParseError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> ExitCode {
        ExitCode::InvalidArguments
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
        if let Some(ctx) = self.as_fail().downcast_ref::<Context<CliParseError>>() {
            if let Some(ref usage) = ctx.get_context().usage {
                return Some(usage);
            }
        }
        None
    }
}

#[derive(Fail, Debug)]
#[fail(display = "command `{}` is not yet implemented", name)]
pub(crate) struct CommandUnimplementedError {
    pub(crate) name: String,
}

impl CommandUnimplementedError {
    pub(crate) fn new(cmd_name: &str) -> Self {
        CommandUnimplementedError {
            name: cmd_name.to_string(),
        }
    }
}

impl NotionFail for CommandUnimplementedError {
    fn is_user_friendly(&self) -> bool {
        true
    }
    fn exit_code(&self) -> ExitCode {
        ExitCode::NotYetImplemented
    }
}
