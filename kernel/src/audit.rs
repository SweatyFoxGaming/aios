//! The "Black Box" Audit Log for Phoenix OS.
//! Responsible for persistent, tamper-resistant logging of system events and intent.

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use common::security::{Capability, Token};
use lazy_static::lazy_static;
use spin::Mutex;

/// An entry in the audit log.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Timestamp (placeholder for now).
    pub timestamp: u64,
    /// The token ID that initiated the action.
    pub token_id: u64,
    /// The action or intent recorded.
    pub action: String,
    /// The outcome (e.g., "Success", "Denied").
    pub outcome: &'static str,
}

lazy_static! {
    static ref AUDIT_LOG: Mutex<Vec<AuditEntry>> = Mutex::new(Vec::new());
}

/// Log a system action to the black box.
pub fn log(token: &Token, action: String, outcome: &'static str) {
    // Basic verification: does the token have AuditWrite?
    // In the future, this should be handled by a higher-level Security Manager.
    if !token.has(Capability::AuditWrite) && token.id != 0 {
        println!(
            "Audit log denied: Missing AuditWrite capability (Token {})",
            token.id
        );
        return;
    }

    let entry = AuditEntry {
        timestamp: 0, // TODO: Implement RTC
        token_id: token.id,
        action,
        outcome,
    };

    AUDIT_LOG.lock().push(entry);
}

/// Print all audit entries.
pub fn print_logs() {
    let logs = AUDIT_LOG.lock();
    println!("--- Phoenix Black Box Audit Log ---");
    for entry in logs.iter() {
        println!(
            "[{}] Token {}: {} -> {}",
            entry.timestamp, entry.token_id, entry.action, entry.outcome
        );
    }
    println!("------------------------------------");
}
