mod common;
use std::env;
use volta_core::run::get_tool_name;

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
    // Seperate Node/Yarn/Npm/Npx from ActivityKind::Tool to get more detail info in events
    let mut native_args = env::args_os();
    let activity_kind = match get_tool_name(&mut native_args) {
        Ok(exe) => match exe.to_str() {
            Some("node") => ActivityKind::Node,
            Some("yarn") => ActivityKind::Yarn,
            Some("npm") => ActivityKind::Npm,
            Some("npx") => ActivityKind::Npx,
            _ => ActivityKind::Tool,
        },
        Err(_) => ActivityKind::Tool,
    };

    session.add_event_start(activity_kind);

    let result = ensure_layout().and_then(|()| execute_shim(&mut session).into_result());
    match result {
        Ok(()) => {
            session.add_event_end(activity_kind, ExitCode::Success);
            session.exit(ExitCode::Success);
        }
        Err(Error::Tool(code)) => {
            session.add_event_tool_end(activity_kind, code);
            session.exit_tool(code);
        }
        Err(Error::Volta(err)) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            session.add_event_error(activity_kind, &err);
            session.add_event_end(activity_kind, err.exit_code());
            session.exit(ExitCode::ExecutionFailure);
        }
    }
}
