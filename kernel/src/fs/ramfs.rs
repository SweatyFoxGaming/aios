//! `RamFS`: A basic memory-backed file system for Iris.

use crate::fs::FileSystem;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::Mutex;

/// A simple memory-backed file system.
pub struct RamFS {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl RamFS {
    /// Create a new `RamFS` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files: Mutex::new(BTreeMap::new()),
        }
    }
}

impl Default for RamFS {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RamFS {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.files.lock().get(path).cloned()
    }

    fn write(&mut self, path: &str, data: Vec<u8>) -> bool {
        self.files.lock().insert(path.to_string(), data);
        true
    }
}
