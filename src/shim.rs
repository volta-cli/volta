use volta_core::error::report_error;
use volta_core::log::{LogContext, Logger};
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::execute_tool;
use volta_fail::ExitCode;

pub fn main() {
    Logger::init_from_env(LogContext::Shim).expect("Only a single Logger should be initialized");

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
            report_error(env!("CARGO_PKG_VERSION"), &err);
            session.add_event_error(ActivityKind::Tool, &err);
            session.exit(ExitCode::ExecutionFailure);
        }
    }
}
