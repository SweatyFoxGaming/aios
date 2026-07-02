//! Synapse: Zero-Copy Shared-Memory IPC for Phoenix OS.

use crate::println;
use alloc::string::String;
use alloc::string::ToString;
use common::synapse::Message;

/// Represents a shared memory frame.
pub struct SharedFrame {
    pub phys_addr: u64,
    pub size: usize,
}

/// Send a large context window via frame transfer (Zero-Copy).
pub fn transfer_frame(target_task_id: u64, frame: SharedFrame) {
    println!(
        "[Synapse] Zero-Copy Transfer: Frame 0x{:x} -> Task {}",
        frame.phys_addr, target_task_id
    );

    // 1. Simulate unmapping from current task
    println!("[Synapse] Unmapping 0x{:x} from current context...", frame.phys_addr);

    // 2. Simulate mapping into target task
    println!("[Synapse] Mapping 0x{:x} into Task {} context...", frame.phys_addr, target_task_id);

    // 3. Dispatch notification message
    let msg = Message {
        sender: "Kernel",
        target: "UserspaceTask", // Placeholder for actual target lookup
        intent: "SharedFrameAttached".into(),
        payload: Some(frame.phys_addr.to_string()),
        frame: None,
    };
    send(msg);

    println!("[Synapse] Zero-Copy transfer successful.");
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
