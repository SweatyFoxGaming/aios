//! Oracle: System call interface for Phoenix OS.

use crate::println;
use alloc::string::String;
use common::synapse::Message;

/// Handler for system calls initiated from userspace.
#[must_use]
pub fn handle_syscall(id: u64, arg1: u64, arg2: u64) -> u64 {
    println!("[Oracle] Syscall received: ID={id}, Arg1={arg1}, Arg2={arg2}");

    match id {
        1 => {
            // Message dispatch syscall
            let sender = "UserSpace";
            let target = "Kernel";
            let intent = String::from("UserRequest");
            crate::synapse::send(Message::new(sender, target, intent));
            0
        }
        _ => 0xFFFF_FFFF_FFFF_FFFF, // Error
    }
}
