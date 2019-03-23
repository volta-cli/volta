use notion_core::error::{ErrorContext, ErrorReporter};
use notion_core::session::{ActivityKind, Session};
use notion_core::tool::execute_tool;

use notion_fail::ExitCode;

pub fn main() {
    let mut session = Session::new();

    session.add_event_start(ActivityKind::Tool);

    match execute_tool(&mut session) {
        Ok(status) if status.success() => {
            session.add_event_end(ActivityKind::Tool, ExitCode::Success);
            session.exit(ExitCode::Success);
        }
        Ok(status) => {
            // ISSUE (#36): if None, in unix, find out the signal
            let code = status.code().unwrap_or(1);
            session.add_event_tool_end(ActivityKind::Tool, code);
            session.exit_tool(code);
        }
        Err(err) => {
            ErrorReporter::new().report(ErrorContext::Shim, &err);
            session.add_event_error(ActivityKind::Tool, &err);
            session.exit(ExitCode::ExecutionFailure);
        }
    }
}
