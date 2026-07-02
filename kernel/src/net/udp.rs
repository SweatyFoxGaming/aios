//! Aether: Ethernet, IPv4, and UDP stack for Phoenix OS.

use crate::println;
use alloc::vec::Vec;

/// A simple UDP packet.
pub struct UdpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: Vec<u8>,
}

/// Process an incoming packet (Simulated).
pub fn process_packet(data: &[u8]) {
    println!("[Aether] Processing incoming packet ({} bytes)", data.len());

    if data.len() > 20 {
        println!("[Aether] Detected IPv4 Payload");
        if data.len() > 28 {
            println!("[Aether] Detected UDP Datagram");
        }
    }
}

/// Send a UDP packet.
pub fn send_udp(packet: UdpPacket, dst_ip: [u8; 4]) {
    println!("[Aether] Sending UDP to {}.{}.{}.{}:{} ({} bytes)",
        dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3], packet.dst_port, packet.payload.len());
}
