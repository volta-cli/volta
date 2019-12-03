use volta_core::error::report_error;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_fail::ExitCode;
use volta_migrate::run_migration;

pub fn main() {
    Logger::init(LogContext::Migration, LogVerbosity::Default)
        .expect("Only a single Logger should be initialized");

    let exit_code = match run_migration() {
        Ok(()) => ExitCode::Success,
        Err(err) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            err.exit_code()
        }
    };

    exit_code.exit();
}
