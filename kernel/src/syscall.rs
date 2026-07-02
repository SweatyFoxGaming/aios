//! Oracle: System call interface for Phoenix OS.

use crate::println;
use alloc::string::String;
use common::security::Token;
use common::synapse::Message;

/// Syscall identifiers.
pub mod ids {
    /// Send a message via Synapse.
    pub const SEND_MSG: u64 = 1;
    /// Read from a file.
    pub const READ_FILE: u64 = 2;
    /// Write to a file.
    pub const WRITE_FILE: u64 = 3;
    /// Log to audit system.
    pub const LOG: u64 = 4;
    /// Exit process.
    pub const EXIT: u64 = 5;
}

/// Handler for system calls initiated from userspace.
#[must_use]
pub fn handle_syscall(id: u64, arg1: u64, arg2: u64) -> u64 {
    // In a real system, these arguments would be pointers to structures or buffers.
    // For now, we simulate basic functionality.

    match id {
        ids::SEND_MSG => {
            let sender = "UserSpace";
            let target = "Kernel";
            let intent = String::from("UserRequest");
            crate::synapse::send(Message::new(sender, target, intent));
            0
        }
        ids::READ_FILE => {
            println!("[Oracle] Syscall: READ_FILE (Handle: {arg1})");
            0
        }
        ids::WRITE_FILE => {
            println!("[Oracle] Syscall: WRITE_FILE (Handle: {arg1})");
            0
        }
        ids::LOG => {
            println!("[Oracle] Syscall: LOG");
            // Create a temporary token for logging (in reality, this would be the process's token)
            let mut temp_token = Token::empty(100);
            temp_token.grant(common::security::Capability::AuditWrite);
            crate::audit::log(&temp_token, String::from("Manual log entry"), "Success");
            0
        }
        ids::EXIT => {
            println!("[Oracle] Syscall: EXIT (Status: {arg1})");
            // Here we would terminate the current process
            0
        }
        _ => {
            println!("[Oracle] Unknown Syscall: ID={id}");
            0xFFFF_FFFF_FFFF_FFFF
        }
    }
}
