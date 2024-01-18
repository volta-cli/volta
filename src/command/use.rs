use crate::command::Command;
use volta_core::error::{ErrorKind, ExitCode, Fallible};
use volta_core::session::{ActivityKind, Session};

// NOTE: These use the same text as the `long_about` in crate::cli.
//       It's hard to abstract since it's in an attribute string.

pub(crate) const USAGE: &str = "The subcommand `use` is deprecated.

    To install a tool in your toolchain, use `volta install`.
    To pin your project's runtime or package manager, use `volta pin`.
";

const ADVICE: &str = "
    To install a tool in your toolchain, use `volta install`.
    To pin your project's runtime or package manager, use `volta pin`.
";

#[derive(clap::Args)]
pub(crate) struct Use {
    #[allow(dead_code)]
    anything: Vec<String>, // Prevent Clap argument errors when invoking e.g. `volta use node`
}

impl Command for Use {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Help);
        let result = Err(ErrorKind::DeprecatedCommandError {
            command: "use".to_string(),
            advice: ADVICE.to_string(),
        }
        .into());
        session.add_event_end(ActivityKind::Help, ExitCode::InvalidArguments);
        result
    }
}
