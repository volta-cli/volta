use volta_core::error::{report_error, ExitCode};
use volta_core::layout::volta_home;
use volta_core::log::{LogContext, LogVerbosity, Logger};
use volta_migrate::run_migration;

pub fn main() {
    Logger::init(LogContext::Migration, LogVerbosity::Default)
        .expect("Only a single Logger should be initialized");

    // In order to migrate the existing Volta directory while avoiding unconditional changes to the user's system,
    // the Homebrew formula runs volta-migrate with `--no-create` flag in the post-install phase.
    let no_create = matches!(std::env::args_os().nth(1), Some(flag) if flag == "--no-create");
    if no_create && !volta_home().map_or(false, |home| home.root().exists()) {
        ExitCode::Success.exit();
    }

    let exit_code = match run_migration() {
        Ok(()) => ExitCode::Success,
        Err(err) => {
            report_error(env!("CARGO_PKG_VERSION"), &err);
            err.exit_code()
        }
    };

    exit_code.exit();
}
