//! Userspace process management for Phoenix OS.

use crate::println;
use alloc::vec::Vec;

/// Represents a userspace process.
pub struct Process {
    /// Unique process identifier.
    pub id: u64,
    /// Friendly name of the process.
    pub name: &'static str,
    /// Process executable code (placeholder).
    pub code: Vec<u8>,
}

/// Loads a userspace process from a byte buffer (simulated ELF loader).
pub fn load(name: &'static str, code: Vec<u8>) {
    println!("[Process] Loading userspace process: {}", name);

    // In a real system, we would:
    // 1. Parse ELF header
    // 2. Create new page table
    // 3. Map code and data segments
    // 4. Create a new task and add it to the scheduler

    let process = Process { id: 1, name, code };

    println!(
        "[Process] Process {} loaded successfully (Code Size: {} bytes).",
        process.name,
        process.code.len()
    );
}

/// Executes a syscall from the current userspace context.
pub fn execute_syscall(id: u64, arg1: u64, arg2: u64) -> u64 {
    crate::syscall::handle_syscall(id, arg1, arg2)
}
