//! Oracle: System call interface for Phoenix OS.

use crate::println;
use alloc::string::String;
use common::security::{Capability, Token};
use common::synapse::Message;

/// Syscall identifiers.
pub mod ids {
    /// Send a Synapse message.
    pub const SEND_MSG: u64 = 1;
    /// Read a file from Iris VFS.
    pub const READ_FILE: u64 = 2;
    /// Write a file to Iris VFS.
    pub const WRITE_FILE: u64 = 3;
    /// Log an event to the audit log.
    pub const LOG: u64 = 4;
    /// Terminate the current process.
    pub const EXIT: u64 = 5;
}

/// Handler for system calls initiated from userspace.
#[must_use]
pub fn handle_syscall(id: u64, arg1: u64, _arg2: u64) -> u64 {
    // Simulated token discovery (in a real system, retrieved from task context)
    let mut caller_token = Token::empty(101);
    // For testing, we grant some capabilities
    caller_token.grant(Capability::AuditWrite);

    match id {
        ids::SEND_MSG => {
            if !caller_token.has(Capability::ServiceRegister) {
                println!("[Oracle] Access Denied: SEND_MSG requires ServiceRegister capability");
                return 1;
            }
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
            if !caller_token.has(Capability::AuditWrite) {
                println!("[Oracle] Access Denied: LOG requires AuditWrite capability");
                return 1;
            }
            println!("[Oracle] Syscall: LOG (Secure)");
            crate::audit::log(&caller_token, String::from("Manual log entry"), "Success");
            0
        }
        ids::EXIT => {
            println!("[Oracle] Syscall: EXIT (Status: {arg1})");
            0
        }
        _ => {
            println!("[Oracle] Unknown Syscall: ID={id}");
            0xFFFF_FFFF_FFFF_FFFF
        }
    }
}
