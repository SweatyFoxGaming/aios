//! Synapse: Structured IPC for Phoenix OS.

use alloc::string::String;

/// Represents a physical memory frame owned by a Synapse message.
#[derive(Debug, Clone)]
pub struct SynapseFrame {
    /// Physical start address of the frame.
    pub phys_addr: u64,
    /// Size of the data in the frame.
    pub size: usize,
}

/// Emotional metadata for agent communication (Prosody).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prosody {
    /// Neutral, factual delivery.
    Calm,
    /// High priority, alert delivery.
    Urgent,
    /// Soft, supportive delivery.
    Empathetic,
    /// Suggests background processing or "pondering".
    Thinking,
}

/// The "Dialect" or domain of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// Human-to-AI or AI-to-Human natural language.
    NaturalLanguage,
    /// Inter-agent cognitive reasoning or state sharing.
    Cognitive,
    /// Low-level system telemetry and hardware signals.
    System,
}

/// Represents a message sent over the Synapse bus.
#[derive(Debug, Clone)]
pub struct Message {
    /// The sender of the message.
    pub sender: &'static str,
    /// The target recipient.
    pub target: &'static str,
    /// The type/intent of the message.
    pub intent: String,
    /// The structured payload.
    pub payload: Option<String>,
    /// The emotional delivery style.
    pub prosody: Prosody,
    /// The domain of the message.
    pub dialect: Dialect,
    /// Optional zero-copy data frame.
    pub frame: Option<SynapseFrame>,
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
            prosody: Prosody::Calm,
            dialect: Dialect::NaturalLanguage,
            frame: None,
        }
    }

    /// Attach a zero-copy frame to the message.
    #[must_use]
    pub const fn with_frame(mut self, phys_addr: u64, size: usize) -> Self {
        self.frame = Some(SynapseFrame { phys_addr, size });
        self
    }
}
