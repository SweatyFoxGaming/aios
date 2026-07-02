//! Synapse: Zero-Copy Shared-Memory IPC for Phoenix OS.

use crate::println;
use alloc::string::String;
use common::synapse::Message;

/// Represents a shared memory frame.
pub struct SharedFrame {
    pub phys_addr: u64,
    pub size: usize,
}

/// Send a large context window via frame transfer (Zero-Copy).
pub fn transfer_frame(target_task_id: u64, frame: SharedFrame) {
    println!("[Synapse] Zero-Copy Transfer: Frame 0x{:x} -> Task {}",
        frame.phys_addr, target_task_id);

    // In a real implementation:
    // 1. Unmap frame from current task's page table.
    // 2. Map frame into target task's page table.
    // 3. Send a Synapse message with the new virtual address.
}

/// Send a standard message.
pub fn send(message: Message) {
    println!("[Synapse] Dispatching: {} -> {} ({})",
        message.sender, message.target, message.intent);
}

/// Debug the Synapse bus.
pub fn debug_bus() {
    println!("--- Synapse Neural Bus ---");
    println!("State: Active");
    println!("Mode: Hybrid (Message / Shared-Memory)");
    println!("--------------------------");
}
