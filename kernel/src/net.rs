//! TCP/IP stack for Phoenix OS.

use crate::println;
use alloc::vec::Vec;

/// Represents a network interface.
pub struct NetworkInterface {
    pub name: &'static str,
    pub ip_addr: [u8; 4],
}

/// Initialize the networking stack.
pub fn init() {
    println!("[Aether] Initializing TCP/IP Stack...");

    let eth0 = NetworkInterface {
        name: "eth0",
        ip_addr: [192, 168, 1, 10],
    };

    println!("[Aether] Interface {} initialized with IP: {}.{}.{}.{}",
        eth0.name, eth0.ip_addr[0], eth0.ip_addr[1], eth0.ip_addr[2], eth0.ip_addr[3]);
}

/// Send a packet over the network.
pub fn send_packet(data: &[u8]) {
    println!("[Aether] Sending packet ({} bytes)...", data.len());
}
