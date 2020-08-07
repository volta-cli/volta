use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::path::Path;

use crate::style::progress_spinner;
use fs2::FileExt;
use log::debug;

const LOCK_FILE: &str = "volta.lock";

/// An RAII implementation of an exclusive lock on the Volta directory. When this falls out of scope,
/// the lock will be unlocked.
pub struct VoltaLock {
    inner: File,
}

impl VoltaLock {
    pub fn acquire(volta_home: &Path) -> std::io::Result<Self> {
        let path = volta_home.join(LOCK_FILE);
        debug!("Acquiring lock on Volta directory: {}", path.display());

        let file = OpenOptions::new().write(true).create(true).open(path)?;
        // First we try to lock the file without blocking. If that fails, then we show a spinner
        // and block until the lock completes.
        if file.try_lock_exclusive().is_err() {
            let spinner = progress_spinner("Waiting for file lock on Volta directory");
            // Note: Blocks until the file can be locked
            let lock_result = file.lock_exclusive();
            spinner.finish_and_clear();
            lock_result?;
        }

        Ok(Self { inner: file })
    }
}

impl Drop for VoltaLock {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.inner.unlock();
    }
}
