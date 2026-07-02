//! Iris: Semantic Virtual File System for Phoenix OS.

use alloc::string::String;
use alloc::vec::Vec;

/// Type of file system node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// A standard file.
    File,
    /// A directory.
    Directory,
    /// A semantic link (JARVIS-specific).
    SemanticLink,
}

/// A node in the virtual file system.
pub struct VfsNode {
    /// Name of the node.
    pub name: String,
    /// Type of the node.
    pub node_type: NodeType,
    /// Metadata tags (Semantic VFS).
    pub tags: Vec<String>,
}

/// Trait representing a file system implementation.
pub trait FileSystem: Send + Sync {
    /// Read a file by name.
    fn read(&self, path: &str) -> Option<Vec<u8>>;
    /// Write a file by name.
    fn write(&mut self, path: &str, data: Vec<u8>) -> bool;
}

pub mod ramfs;
pub mod phoenixfs;
pub mod shell;
