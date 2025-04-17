use std::sync::Arc;

use wasmtime_wasi::{FileInputStream, StdinStream};

pub struct FileStdinStream {
    file: Arc<File>,
}

impl FileStdinStream {
    pub fn new(file: File) -> Self {
        Self {
            file: Arc::new(file),
        }
    }
}

impl StdinStream for FileStdinStream {
    fn stream(&self) -> Box<dyn wasmtime_wasi::InputStream> {
        Box::new(FileInputStream::new(Arc::clone(&self.file), 0))
    }

    fn isatty(&self) -> bool {
        false
    }
}
