use magnus::{value::InnerValue, value::Opaque, RString, Ruby};
use std::io;

pub struct OutputLimitedBuffer {
    buffer: Opaque<RString>,
    /// The maximum number of bytes that can be written to the output stream buffer.
    capacity: usize,
}

impl OutputLimitedBuffer {
    #[must_use]
    pub fn new(buffer: Opaque<RString>, capacity: usize) -> Self {
        Self { buffer, capacity }
    }
}

impl io::Write for OutputLimitedBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        /// Append a buffer to the string and truncate when hitting the capacity.
        /// We return the input buffer size regardless of whether we truncated or not to avoid a panic.
        let ruby = Ruby::get().unwrap();

        let mut inner_buffer = self.buffer.get_inner_with(&ruby);

        if buf.is_empty() {
            return Ok(buf.len());
        }

        if inner_buffer.len() + buf.len() > self.capacity {
            let portion = self.capacity - inner_buffer.len();
            inner_buffer.write(&buf[0..portion])?
        } else {
            inner_buffer.write(buf)?
        };

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let ruby = Ruby::get().unwrap();

        self.buffer.get_inner_with(&ruby).flush()
    }
}
