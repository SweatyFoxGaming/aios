//! Synapse: The intelligence communication highway for Phoenix OS.

use alloc::collections::VecDeque;
use alloc::string::ToString;
use common::synapse::Message;
use lazy_static::lazy_static;
use spin::Mutex;

/// Maximum number of messages in the buffer.
const MAX_BACKLOG: usize = 128;

lazy_static! {
    static ref SYNAPSE_BUS: Mutex<VecDeque<Message>> =
        Mutex::new(VecDeque::with_capacity(MAX_BACKLOG));
}

/// Send a message over the Synapse bus.
pub fn send(msg: Message) {
    let mut bus = SYNAPSE_BUS.lock();
    if bus.len() >= MAX_BACKLOG {
        let _ = bus.pop_front();
    }

    // Log the intent to the neural bus for visibility
    let frame_info = msg
        .frame
        .as_ref()
        .map_or_else(alloc::string::String::new, |f| {
            alloc::format!(" [Zero-Copy Frame: 0x{:x}, {} bytes]", f.phys_addr, f.size)
        });

    crate::events::publish(
        "Synapse: ".to_string()
            + msg.sender
            + " -> "
            + msg.target
            + " ["
            + &msg.intent
            + "]"
            + &frame_info,
        0.3,
    );

    bus.push_back(msg);
}

/// Peek at the last message on the bus.
#[must_use]
pub fn peek_last() -> Option<Message> {
    SYNAPSE_BUS.lock().back().cloned()
}

/// Print current bus status.
pub fn debug_bus() {
    let bus = SYNAPSE_BUS.lock();
    crate::println!("--- Synapse IPC Backlog ---");
    for (i, msg) in bus.iter().enumerate() {
        let frame_status = if msg.frame.is_some() { "[ZC]" } else { "" };
        crate::println!(
            "[{}] {} -> {}: {} {}",
            i,
            msg.sender,
            msg.target,
            msg.intent,
            frame_status
        );
    }
    crate::println!("---------------------------");
}
