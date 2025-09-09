use bytes::Bytes;
use magnus::{
    value::{InnerValue, Opaque, ReprValue},
    RString, Ruby,
};
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWrite;
use wasmtime_wasi::cli::{IsTerminal, StdoutStream};
use wasmtime_wasi::p2::{OutputStream, Pollable, StreamError, StreamResult};

/// A buffer that limits the number of bytes that can be written to it.
/// If the buffer is full, it will truncate the data.
/// Is used in the buffer implementations of stdout and stderr in `WasiP1Ctx` and `WasiCtxBuilder`.
pub struct OutputLimitedBuffer {
    inner: Arc<Mutex<OutputLimitedBufferInner>>,
}

// No support for WASI P3, yet.
impl AsyncWrite for OutputLimitedBuffer {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::task::Poll::Ready(Ok(0))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl OutputLimitedBuffer {
    /// Creates a new [OutputLimitedBuffer] with the given underlying buffer
    /// and capacity.
    pub fn new(buffer: Opaque<RString>, capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(OutputLimitedBufferInner::new(buffer, capacity))),
        }
    }
}

impl Clone for OutputLimitedBuffer {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl StdoutStream for OutputLimitedBuffer {
    fn p2_stream(&self) -> Box<dyn OutputStream> {
        let cloned = self.clone();
        Box::new(cloned)
    }

    fn async_stream(&self) -> Box<dyn AsyncWrite + Send + Sync> {
        let cloned = self.clone();
        Box::new(cloned)
    }
}

impl IsTerminal for OutputLimitedBuffer {
    fn is_terminal(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl Pollable for OutputLimitedBuffer {
    async fn ready(&mut self) {}
}

impl OutputStream for OutputLimitedBuffer {
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

struct OutputLimitedBufferInner {
    buffer: Opaque<RString>,
    /// The maximum number of bytes that can be written to the output stream buffer.
    capacity: usize,
}

impl OutputLimitedBufferInner {
    #[must_use]
    pub fn new(buffer: Opaque<RString>, capacity: usize) -> Self {
        Self { buffer, capacity }
    }
}

impl OutputLimitedBufferInner {
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
            inner_buffer
                .write(buf)
                .map_err(|e| StreamError::trap(&e.to_string()))?;
        } else {
            let portion = self.capacity - inner_buffer.len();
            inner_buffer
                .write(&buf[0..portion])
                .map_err(|e| StreamError::trap(&e.to_string()))?;
        };

        Ok(())
    }
}
