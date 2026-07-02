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
    /// Ticks since boot.
    pub timestamp: u64,
    /// The token ID that initiated the action.
    pub token_id: u64,
    /// The action or intent recorded.
    pub action: String,
    /// The outcome (e.g., "Success", "Denied").
    pub outcome: &'static str,
    /// Cryptographic hash linking to the previous entry (The Immutable Chain).
    pub chain_hash: u64,
}

lazy_static! {
    static ref AUDIT_LOG: Mutex<Vec<AuditEntry>> = Mutex::new(Vec::new());
}

/// Simple XOR-based rolling hash for early ledger integrity.
fn calculate_hash(prev_hash: u64, action: &str) -> u64 {
    let mut hash = prev_hash;
    for byte in action.as_bytes() {
        hash = hash.wrapping_add(u64::from(*byte)).rotate_left(7) ^ 0xDEAD_BEEF_CAFE_BABE;
    }
    hash
}

/// Log a system action to the black box.
pub fn log(token: &Token, action: String, outcome: &'static str) {
    // Basic verification: does the token have AuditWrite?
    if !token.has(Capability::AuditWrite) && token.id != 0 {
        println!(
            "Audit log denied: Missing AuditWrite capability (Token {})",
            token.id
        );
        return;
    }

    let mut logs = AUDIT_LOG.lock();
    let prev_hash = logs.last().map_or(0xFEED_FACE_CAFE_BEEF, |e| e.chain_hash);
    let new_hash = calculate_hash(prev_hash, &action);

    let entry = AuditEntry {
        timestamp: crate::arch::x86_64::time::get_uptime(),
        token_id: token.id,
        action,
        outcome,
        chain_hash: new_hash,
    };

    logs.push(entry);
}

/// Print all audit entries.
pub fn print_logs() {
    let logs = AUDIT_LOG.lock();
    println!("--- Phoenix Black Box Immutable Audit Log ---");
    for entry in logs.iter() {
        println!(
            "[{}] [Hash: {:x}] Token {}: {} -> {}",
            entry.timestamp, entry.chain_hash, entry.token_id, entry.action, entry.outcome
        );
    }
    println!("----------------------------------------------");
}
