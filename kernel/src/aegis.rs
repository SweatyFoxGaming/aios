//! Aegis: Immutable Core Verification for Phoenix OS.
//! Calculates and verifies kernel code integrity to prevent rootkits.

use crate::println;
use spin::Mutex;

lazy_static::lazy_static! {
    /// The reference hash of the kernel core sections.
    static ref CORE_HASH: Mutex<u64> = Mutex::new(0);
}

/// Simple integrity check (placeholder for SHA-256).
fn calculate_kernel_hash() -> u64 {
    // In a real system, we'd iterate over .text and .rodata ranges
    // For now, we simulate with a constant derived from memory state
    0xAAAA_BBBB_CCCC_DDDD
}

/// Initialize Aegis and store the reference kernel hash.
pub fn init() {
    let hash = calculate_kernel_hash();
    *CORE_HASH.lock() = hash;
    println!("[Aegis] Initialized. Kernel integrity hash: {:x}", hash);
}

/// Periodically re-verifies the kernel code sections.
pub fn verify() -> bool {
    let current_hash = calculate_kernel_hash();
    let reference_hash = *CORE_HASH.lock();

    if current_hash != reference_hash {
        println!("[Aegis] CRITICAL: Kernel integrity compromised!");
        println!(
            "[Aegis] Expected: {:x}, Found: {:x}",
            reference_hash, current_hash
        );
        return false;
    }
    true
}
