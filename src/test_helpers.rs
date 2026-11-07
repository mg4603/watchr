#[cfg(test)]
use std::io::{self, Result, Write};

pub(crate) struct FailingWriter {
    pub(crate) remaining: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() > self.remaining {
            Err(io::Error::other("write failed"))
        } else {
            self.remaining -= buf.len();
            Ok(buf.len())
        }
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub(crate) struct FlushFailingWriter;
impl Write for FlushFailingWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Err(io::Error::other("flush failed"))
    }
}
