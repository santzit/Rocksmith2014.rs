use std::sync::Mutex;

/// A simple pool of reusable `Vec<u8>` buffers to reduce allocations.
pub struct MemoryStreamPool {
    pool: Mutex<Vec<Vec<u8>>>,
}

impl MemoryStreamPool {
    /// Creates a new empty pool.
    pub const fn new() -> Self {
        Self {
            pool: Mutex::new(Vec::new()),
        }
    }

    /// Takes a buffer from the pool, or creates a new one.
    pub fn rent(&self) -> Vec<u8> {
        self.pool.lock().unwrap().pop().unwrap_or_default()
    }

    /// Returns a buffer to the pool after clearing it.
    pub fn return_buffer(&self, mut buf: Vec<u8>) {
        buf.clear();
        self.pool.lock().unwrap().push(buf);
    }
}

impl Default for MemoryStreamPool {
    fn default() -> Self {
        Self::new()
    }
}
