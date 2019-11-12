use volta_core::error::report_error;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_core::run::execute_tool;
use volta_core::session::{ActivityKind, Session};
use volta_core::signal::setup_signal_handler;
use volta_fail::ExitCode;

pub(super) fn run_shim() {
    Logger::init(LogContext::Shim, LogVerbosity::Default)
        .expect("Only a single Logger should be initialized");
    setup_signal_handler();

    let mut session = Session::init();

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
