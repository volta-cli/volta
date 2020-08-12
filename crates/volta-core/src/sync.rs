use std::fs::{File, OpenOptions};
use std::marker::PhantomData;
use std::ops::Drop;
use std::sync::Mutex;

use crate::error::{Context, ErrorKind, Fallible};
use crate::layout::volta_home;
use crate::style::progress_spinner;
use fs2::FileExt;
use lazy_static::lazy_static;
use log::debug;

lazy_static! {
    static ref LOCK_STATE: Mutex<Option<LockState>> = Mutex::new(None);
}

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
