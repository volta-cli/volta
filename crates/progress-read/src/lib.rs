//! This crate provides an adapter for the `std::io::Read` trait to
//! allow reporting incremental progress to a callback function.

use std::io::{self, Read, Seek, SeekFrom};

/// A reader that reports incremental progress while reading.
pub struct ProgressRead<R: Read, T, F: FnMut(&T, usize) -> T> {
    source: R,
    accumulator: T,
    progress: F,
}

impl<R: Read, T, F: FnMut(&T, usize) -> T> Read for ProgressRead<R, T, F> {
    /// Read some bytes from the underlying reader into the specified buffer,
    /// and report progress to the progress callback. The progress callback is
    /// passed the current value of the accumulator as its first argument and
    /// the number of bytes read as its second argument. The result of the
    /// progress callback is stored as the updated value of the accumulator,
    /// to be passed to the next invocation of the callback.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.source.read(buf)?;
        let new_accumulator = {
            let progress = &mut self.progress;
            progress(&self.accumulator, len)
        };
        self.accumulator = new_accumulator;
        Ok(len)
    }
}

impl<R: Read, T, F: FnMut(&T, usize) -> T> ProgressRead<R, T, F> {
    /// Construct a new progress reader with the specified underlying reader,
    /// initial value for an accumulator, and progress callback.
    pub fn new(source: R, init: T, progress: F) -> ProgressRead<R, T, F> {
        ProgressRead {
            source,
            accumulator: init,
            progress,
        }
    }
}

impl<R: Read + Seek, T, F: FnMut(&T, usize) -> T> Seek for ProgressRead<R, T, F> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.source.seek(pos)
    }
}
