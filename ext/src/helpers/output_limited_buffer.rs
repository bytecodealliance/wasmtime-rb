use bytes::Bytes;
use magnus::{
    value::{InnerValue, Opaque, ReprValue},
    RString, Ruby,
};
use std::io::Write;
use std::sync::{Arc, Mutex};
use wasmtime_wasi::{OutputStream, Pollable, StdoutStream, StreamError, StreamResult};

pub struct WritePipe {
    inner: Arc<Mutex<OutputLimitedBuffer>>,
}

impl WritePipe {
    pub fn new(inner: OutputLimitedBuffer) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl Clone for WritePipe {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl StdoutStream for WritePipe {
    fn stream(&self) -> Box<dyn wasmtime_wasi::OutputStream> {
        let cloned = self.clone();
        Box::new(cloned)
    }

    fn isatty(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl Pollable for WritePipe {
    async fn ready(&mut self) {}
}

impl OutputStream for WritePipe {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        let mut stream = self.inner.lock().expect("Should be only writer");
        stream.write(&bytes)
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        let mut stream = self.inner.lock().expect("Should be only writer");
        stream.check_write()
    }
}

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

impl OutputLimitedBuffer {
    fn check_write(&mut self) -> StreamResult<usize> {
        Ok(usize::MAX)
    }

    fn write(&mut self, buf: &[u8]) -> StreamResult<()> {
        // Append a buffer to the string and truncate when hitting the capacity.
        // We return the input buffer size regardless of whether we truncated or not to avoid a panic.
        let ruby = Ruby::get().unwrap();

        let mut inner_buffer = self.buffer.get_inner_with(&ruby);

        // Handling frozen case here is necessary because magnus does not check if a string is frozen before writing to it.
        let is_frozen = inner_buffer.as_value().is_frozen();
        if is_frozen {
            return Err(StreamError::trap("Cannot write to a frozen buffer."));
        }

        if buf.is_empty() {
            return Ok(());
        }

        if inner_buffer
            .len()
            .checked_add(buf.len())
            .is_some_and(|val| val < self.capacity)
        {
            let amount_written = inner_buffer
                .write(buf)
                .map_err(|e| StreamError::trap(&e.to_string()))?;
            if amount_written < buf.len() {
                return Ok(());
            }
        } else {
            let portion = self.capacity - inner_buffer.len();
            let amount_written = inner_buffer
                .write(&buf[0..portion])
                .map_err(|e| StreamError::trap(&e.to_string()))?;
            if amount_written < portion {
                return Ok(());
            }
        };

        Ok(())
    }
}
