use std::io::{self, Read};

pub struct ProgressRead<R: Read, T, F: FnMut(&T, usize) -> T> {
    source: R,
    accumulator: T,
    progress: F
}

impl<R: Read, T, F: FnMut(&T, usize) -> T> Read for ProgressRead<R, T, F> {
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
    pub fn new(source: R, init: T, progress: F) -> ProgressRead<R, T, F> {
        ProgressRead { source, accumulator: init, progress }
    }
}
