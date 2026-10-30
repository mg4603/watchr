#[cfg(test)]
use std::io::{Error, Result, Write};
pub(crate) struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        Err(Error::other("write failed"))
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
