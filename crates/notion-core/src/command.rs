use std::ffi::OsStr;
use std::process::Command;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        pub fn create_command<E>(exe: E) -> Command
        where
            E: AsRef<OsStr>
        {
            // Several of the node utilities are implemented as `.bat` or `.cmd` files
            // When executing those files with `Command`, we need to call them with:
            //    cmd.exe /C <COMMAND> <ARGUMENTS>
            // Instead of: <COMMAND> <ARGUMENTS>
            // See: https://github.com/rust-lang/rust/issues/42791 For a longer discussion
            let mut command = Command::new("cmd.exe");
            command.arg("/C");
            command.arg(exe);
            command
        }
    } else {
        pub fn create_command<E>(exe: E) -> Command
        where
            E: AsRef<OsStr>
        {
            Command::new(exe)
        }
    }
}
