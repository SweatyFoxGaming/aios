//! Synapse: Structured IPC for Phoenix OS.

use alloc::string::String;

/// Represents a message sent over the Synapse bus.
#[derive(Debug, Clone)]
pub struct Message {
    /// The sender of the message.
    pub sender: &'static str,
    /// The target recipient.
    pub target: &'static str,
    /// The type/intent of the message.
    pub intent: String,
    /// The structured payload (placeholder for now).
    pub payload: Option<String>,
}

impl Message {
    /// Create a new Synapse message.
    #[must_use]
    pub const fn new(sender: &'static str, target: &'static str, intent: String) -> Self {
        Self {
            sender,
            target,
            intent,
            payload: None,
        }
    }
}
