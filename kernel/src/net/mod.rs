//! Aether Networking Stack for Phoenix OS.

pub mod udp;

use crate::println;

/// Initialize the networking stack.
pub fn init() {
    println!("[Aether] Transcendent Networking (UDP/IP/Ethernet) initialized.");
}
