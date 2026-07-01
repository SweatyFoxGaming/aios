/// Error types for Phoenix OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoenixError {
    /// An unknown error occurred.
    Unknown,
    /// System is out of memory.
    OutOfMemory,
    /// The provided address is invalid.
    InvalidAddress,
    /// An I/O error occurred.
    IoError,
    /// The requested feature is not implemented.
    NotImplemented,
}

/// Result type for Phoenix OS.
pub type PhoenixResult<T> = core::result::Result<T, PhoenixError>;
