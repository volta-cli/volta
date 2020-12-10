mod common;

use common::{ensure_layout, Error, IntoResult};
use volta_core::error::{report_error, ExitCode};
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_core::run::execute_shim;
use volta_core::session::{ActivityKind, Session};
use volta_core::signal::setup_signal_handler;

pub fn main() {
    Logger::init(LogContext::Shim, LogVerbosity::Default)
        .expect("Only a single Logger should be initialized");
    setup_signal_handler();

    let mut session = Session::init();
    session.add_event_start(ActivityKind::Tool);

    let result = ensure_layout().and_then(|()| execute_shim(&mut session).into_result());
    match result {
        Ok(()) => {
            session.add_event_end(ActivityKind::Tool, ExitCode::Success);
            session.exit(ExitCode::Success);
        }
        Err(Error::Tool(code)) => {
            session.add_event_tool_end(ActivityKind::Tool, code);
            session.exit_tool(code);
        }
        Err(Error::Volta(err)) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            session.add_event_error(ActivityKind::Tool, &err);
            session.add_event_end(ActivityKind::Tool, err.exit_code());
            session.exit(ExitCode::ExecutionFailure);
        }
    }
}
