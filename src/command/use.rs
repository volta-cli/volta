use structopt::StructOpt;

use crate::command::Command;
use jetson_core::error::ErrorDetails;
use jetson_core::session::{ActivityKind, Session};
use jetson_fail::{ExitCode, Fallible};

// NOTE: These use the same text as the `long_about` in crate::cli.
//       It's hard to abstract since it's in an attribute string.

pub(crate) const USAGE: &'static str = "The subcommand `use` is deprecated.

    To install a tool in your toolchain, use `jetson install`.
    To pin your project's runtime or package manager, use `jetson pin`.
";

const ADVICE: &'static str = "
    To install a tool in your toolchain, use `jetson install`.
    To pin your project's runtime or package manager, use `jetson pin`.
";

#[derive(StructOpt)]
pub(crate) struct Use {
    #[allow(dead_code)]
    anything: Vec<String>, // Prevent StructOpt argument errors when invoking e.g. `jetson use node`
}

impl Command for Use {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Help);
        let result = Err(ErrorDetails::DeprecatedCommandError {
            command: "use".to_string(),
            advice: ADVICE.to_string(),
        }
        .into());
        session.add_event_end(ActivityKind::Help, ExitCode::InvalidArguments);
        result
    }
}
