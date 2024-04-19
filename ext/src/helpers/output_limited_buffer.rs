use magnus::{
    value::{InnerValue, Opaque, ReprValue},
    RString, Ruby,
};
use std::io;
use std::io::ErrorKind;

/// A buffer that limits the number of bytes that can be written to it.
/// If the buffer is full, it will truncate the data.
/// Is used in the buffer implementations of stdout and stderr in `WasiCtx` and `WasiCtxBuilder`.
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
        // Append a buffer to the string and truncate when hitting the capacity.
        // We return the input buffer size regardless of whether we truncated or not to avoid a panic.
        let ruby = Ruby::get().unwrap();

        let mut inner_buffer = self.buffer.get_inner_with(&ruby);

        let is_frozen = inner_buffer.as_value().is_frozen();

        if is_frozen {
            return (Err(io::Error::new(
                ErrorKind::WriteZero,
                "Cannot write to a frozen buffer.",
            )));
            // return Ok(buf.len());
        }

        if buf.is_empty() {
            return Ok(0);
        }

        if inner_buffer
            .len()
            .checked_add(buf.len())
            .is_some_and(|val| val < self.capacity)
        {
            let amount_written = inner_buffer.write(buf)?;
            if amount_written < buf.len() {
                return Ok(amount_written);
            }
        } else {
            let portion = self.capacity - inner_buffer.len();
            let amount_written = inner_buffer.write(&buf[0..portion])?;
            if amount_written < portion {
                return Ok(amount_written);
            }
        };

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let ruby = Ruby::get().unwrap();

        self.buffer.get_inner_with(&ruby).flush()
    }
}
