//! Inter-process locking on the Volta directory
//!
//! To avoid issues where multiple separate invocations of Volta modify the
//! data directory simultaneously, we provide a locking mechanism that only
//! allows a single process to modify the directory at a time.
//!
//! However, within a single process, we may attempt to lock the directory in
//! different code paths. For example, when installing a package we require a
//! lock, however we also may need to install Node, which requires a lock as
//! well. To avoid deadlocks in those situations, we track the state of the
//! lock globally:
//!
//! - If a lock is requested and no locks are active, then we acquire a file
//!   lock on the `volta.lock` file and initialize the state with a count of 1
//! - If a lock already exists, then we increment the count of active locks
//! - When a lock is no longer needed, we decrement the count of active locks
//! - When the last lock is released, we release the file lock and clear the
//!   global lock state.
//!
//! This allows multiple code paths to request a lock and not worry about
//! potential deadlocks, while still preventing multiple processes from making
//! concurrent changes.

use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::ops::Drop;
use std::sync::Mutex;

use crate::error::{Context, ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::style::progress_spinner;
use fs2::FileExt;
use log::debug;
use once_cell::sync::Lazy;

static LOCK_STATE: Lazy<Mutex<Option<LockState>>> = Lazy::new(|| Mutex::new(None));

/// The current state of locks for this process.
///
/// Note: To ensure thread safety _within_ this process, we enclose the
/// state in a Mutex. This Mutex and it's associated locks are separate
/// from the overall process lock and are only used to ensure the count
/// is accurately maintained within a given process.
struct LockState {
    file: File,
    count: usize,
}

const LOCK_FILE: &str = "volta.lock";

/// An RAII implementation of a process lock on the Volta directory. A given Volta process can have
/// multiple active locks, but only one process can have any locks at a time.
///
/// Once all of the `VoltaLock` objects go out of scope, the lock will be released to other
/// processes.
pub struct VoltaLock {
    // Private field ensures that this cannot be created except for with the `acquire()` method
    _private: PhantomData<()>,
}

impl VoltaLock {
    pub fn acquire() -> Fallible<Self> {
        let mut state = LOCK_STATE
            .lock()
            .with_context(|| ErrorKind::LockAcquireError)?;

        // Check if there is an active lock for this process. If so, increment
        // the count of active locks. If not, create a file lock and initialize
        // the state with a count of 1
        match &mut *state {
            Some(inner) => {
                inner.count += 1;
            }
            None => {
                let path = volta_home()?.root().join(LOCK_FILE);
                debug!("Acquiring lock on Volta directory: {}", path.display());

                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
                    .with_context(|| ErrorKind::LockAcquireError)?;
                // First we try to lock the file without blocking. If that fails, then we show a spinner
                // and block until the lock completes.
                if file.try_lock_exclusive().is_err() {
                    let spinner = progress_spinner("Waiting for file lock on Volta directory");
                    // Note: Blocks until the file can be locked
                    let lock_result = file
                        .lock_exclusive()
                        .with_context(|| ErrorKind::LockAcquireError);
                    spinner.finish_and_clear();
                    lock_result?;
                }

                *state = Some(LockState { file, count: 1 });
            }
        }

        Ok(Self {
            _private: PhantomData,
        })
    }
}

impl Drop for VoltaLock {
    fn drop(&mut self) {
        // On drop, decrement the count of active locks. If the count is 1,
        // then this is the last active lock, so instead unlock the file and
        // clear out the lock state.
        if let Ok(mut state) = LOCK_STATE.lock() {
            match &mut *state {
                Some(inner) => {
                    if inner.count == 1 {
                        debug!("Unlocking Volta Directory");
                        let _ = inner.file.unlock();
                        *state = None;
                    } else {
                        inner.count -= 1;
                    }
                }
                None => {
                    debug!("Unexpected unlock of Volta directory when it wasn't locked");
                }
            }
        }
    }
}
