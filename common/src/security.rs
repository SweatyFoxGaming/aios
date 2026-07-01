//! Capability-based security types for Phoenix OS.

/// Represents a specific permission or capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Permission to write to the serial port.
    SerialWrite,
    /// Permission to read from the serial port.
    SerialRead,
    /// Permission to allocate physical memory.
    MemAlloc,
    /// Permission to map virtual memory.
    MemMap,
    /// Permission to manage processes.
    ProcessManage,
    /// Permission to register system services.
    ServiceRegister,
    /// Permission to read hardware identity.
    HardwareInfo,
    /// Permission to write to the audit log.
    AuditWrite,
}

/// A token that grants a set of capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The unique ID of the token.
    pub id: u64,
    /// The list of capabilities granted by this token.
    pub capabilities: [bool; 32], // Bitmask for simplicity
}

impl Token {
    /// Create a new token with no capabilities.
    #[must_use]
    pub const fn empty(id: u64) -> Self {
        Self {
            id,
            capabilities: [false; 32],
        }
    }

    /// Grant a capability to the token.
    pub const fn grant(&mut self, cap: Capability) {
        self.capabilities[cap as usize] = true;
    }

    /// Check if the token has a specific capability.
    #[must_use]
    pub const fn has(&self, cap: Capability) -> bool {
        self.capabilities[cap as usize]
    }
}
